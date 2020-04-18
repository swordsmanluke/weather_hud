use crate::forecast::{Forecast, Forecaster, WeatherFormatter, Weather};
use crate::darksky_client::{DarkSkyClient, DSForecast};
use crate::forecast::Weather::*;
use chrono::{DateTime, Local, TimeZone, Date};
use restson::Error;

pub struct DarkSkyForecaster {
    pub client: Box<dyn DarkSkyClient>
}

impl Forecaster for DarkSkyForecaster {
    fn daily_forecast(&mut self) -> Result<Vec<Forecast>, Error> {
        let forecasts = self.client.forecasts()?.daily.data.into_iter().
                                        map(|f| self.ds_to_forecast(f, true)).
                                        collect();
        Ok(forecasts)
    }

    fn hourly_forecast(&mut self) -> Result<Vec<Forecast>, Error> {
        let forecasts = self.client.forecasts()?.hourly.data.into_iter().
            map(|f| self.ds_to_forecast(f, false)).
            collect();
        Ok(forecasts)
    }

    fn current(&mut self) -> Result<Forecast, Error> {
        let current = self.client.forecasts()?.currently;
        Ok(Forecast::for_time(Sunny,
                              0.0,
                              Some(current.temperature),
                              Some(Local::now())))
    }
}

impl DarkSkyForecaster {
    pub fn new(client: Box<dyn DarkSkyClient>) -> DarkSkyForecaster{
        DarkSkyForecaster { client }
    }

    fn ds_to_forecast(&self, ds_forecast: DSForecast, daily: bool) -> Forecast {
        if daily {
            Forecast::for_date(
                weather(ds_forecast.icon.as_str()),
                ds_forecast.precip_probability.unwrap_or(0.0),
                None,
                Some(date_for(ds_forecast.time))
            )
        } else {
            Forecast::for_time(
                weather(ds_forecast.icon.as_str()),
                ds_forecast.precip_probability.unwrap_or(0.0),
                None,
                Some(time_for(ds_forecast.time))
            )
        }
    }
}

fn weather(icon: &str) -> Weather {
    let resp = match icon {
        "clear-day" | "clear-night" => Sunny,
        "partly-cloudy-day" | "partly-cloudy-night" => PartSun,
        "cloudy" => Cloudy,
        "rain" => Rain,
        "fog" => Fog,
        "snow" => Snow,
        _ => Unknown
    };

    resp
}

fn time_for(seconds: i64) -> DateTime<Local> {
    Local.timestamp_millis(seconds * 1000)
}

fn date_for(seconds: i64) -> Date<Local> {
    time_for(seconds).date()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::darksky_client::{DSForecasts, DSCurrent, DSData};
    use restson::Error::{HttpClientError, SerializeParseError};

    #[test]
    fn parses_daily_results_correctly() {
        let mut dsf = build_forecaster();
        let df = dsf.daily_forecast().unwrap();

        assert_eq!(df.len(), 1);
        for f in df {
            assert_eq!(f.weather, Sunny);
            assert_eq!(f.precip_chance, 0.0);
            assert_eq!(f.temp, None);
            assert_eq!(f.date, Some(Local::today()));
            assert_eq!(f.time, None);
        }
    }

    #[test]
    fn parses_hourly_results_correctly() {
        let mut dsf = build_forecaster();
        let df = dsf.hourly_forecast().unwrap();

        assert_eq!(df.len(), 1);
        for f in df {
            assert_eq!(f.weather, Sunny);
            assert_eq!(f.precip_chance, 0.0);
            assert_eq!(f.temp, None);
            assert_eq!(f.date, None);
            assert!(Local::now().timestamp_millis() - f.time.unwrap().timestamp_millis() < 100);
        }
    }

    #[test]
    fn returns_error() {
        let mut dsf = build_error_forecaster();
        let df = dsf.hourly_forecast();
        assert!(df.is_err())
    }

    fn build_forecaster() -> DarkSkyForecaster {
        let client = MockClient::expect_response(simple_response());
        let mut dsf = DarkSkyForecaster::new(Box::new(client));
        dsf
    }

    fn build_error_forecaster() -> DarkSkyForecaster {
        let client = MockClient::expect_error(HttpClientError);
        let mut dsf = DarkSkyForecaster::new(Box::new(client));
        dsf
    }

    fn simple_response() -> DSForecasts {
        DSForecasts {
            daily: DSData{
                summary: "".to_string(),
                data: vec!(DSForecast {
                time: Local::now().timestamp_millis(),
                icon: "clear-day".to_string(),
                precipProbability: 0.0,
                temperature_high: 66.3,
                temperature_low: 42.5,
            })},
            hourly: DSData{
                summary: "".to_string(),
                data: vec!(DSForecast {
                time: Local::now().timestamp_millis(),
                icon: "clear-day".to_string(),
                precipProbability: 0.0,
                temperature_high: 66.3,
                temperature_low: 42.5,
            })},
            currently: DSCurrent { temperature: 56.2, icon: "sunny".to_string(), precip_probability: 0.0 }
        }
    }

    struct MockClient{
        pub expected_forecasts: Option<DSForecasts>,
        pub expected_error: Option<Error>
    }

    impl MockClient {
        pub fn new(forecasts: Option<DSForecasts>, error: Option<Error>) -> MockClient {
            if forecasts.is_none() && error.is_none() { panic!("forecasts and error cannot both be None!") }
            if forecasts.is_some() && error.is_some() { panic!("forecasts and error cannot both be Some!") }

            MockClient {
                expected_forecasts: forecasts,
                expected_error: error
            }
        }

        pub fn expect_response(forecasts: DSForecasts) -> MockClient {
            MockClient::new(Some(forecasts), None)
        }

        pub fn expect_error(error: Error) -> MockClient {
            MockClient::new(None, Some(error))
        }
    }

    impl DarkSkyClient for MockClient {
        fn forecasts(&mut self) -> Result<DSForecasts, Error> {
            match &self.expected_error {
                Some(e) => Err(HttpClientError),
                None => Ok(self.expected_forecasts.as_ref().unwrap().clone())
            }
        }
    }
}