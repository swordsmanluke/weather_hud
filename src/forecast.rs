use chrono::{Local, Date, DateTime};
use restson::Error;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Weather {
    Sunny,
    PartSun,
    Cloudy,
    Showers,
    Rain,
    Snow,
    Fog,
    Unknown
}

pub trait WeatherFormatter {
    fn format(&self, w: Weather, precip_chance: f32) -> String;
}

pub trait Forecaster {
    fn daily_forecast(&mut self) -> Result<Vec<Forecast>, Error>;
    fn hourly_forecast(&mut self) -> Result<Vec<Forecast>, Error>;
    fn current(&mut self) -> Result<Forecast, Error>;
}

#[derive(Debug, Copy, Clone)]
pub struct Forecast {
    pub weather: Weather,
    pub precip_chance: f32,
    pub temp: Option<f32>,
    pub date: Option<Date<Local>>,
    pub time: Option<DateTime<Local>>,
}

impl Forecast {
    pub fn new(weather: Weather,
               precip_chance: f32,
               temp: Option<f32>,
               date: Option<Date<Local>>,
               time: Option<DateTime<Local>>) -> Forecast {
        Forecast {
            weather,
            precip_chance,
            temp,
            date,
            time,
        }
    }

    pub fn for_date(weather: Weather,
                   precip_chance: f32,
                   temp: Option<f32>,
                   date: Option<Date<Local>>) -> Forecast {
        Forecast::new(
            weather,
            precip_chance,
            temp,
            date,
            None
        )
    }

    pub fn for_time(weather: Weather,
                   precip_chance: f32,
                   temp: Option<f32>,
                   time: Option<DateTime<Local>>) -> Forecast {
        Forecast::new(
            weather,
            precip_chance,
            temp,
            None,
            time
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
}

impl LatLong {
    pub fn to_string(&self) -> String {
        format!("{},{}", self.latitude, self.longitude)
    }
}