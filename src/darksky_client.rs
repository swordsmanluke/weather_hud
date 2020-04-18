use restson::{Error, RestClient, RestPath};
use crate::forecast::LatLong;

const URL_BASE: &str = "https://api.darksky.net/";

pub trait DarkSkyClient {
    fn forecasts(&mut self) -> Result<DSForecasts, Error>;
}

pub struct DarkSkyRestClient {
    pub token: String,
    pub location: LatLong,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DSArgs {
    pub token: String,
    pub location: LatLong
}

// DS_Client objects
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DSForecasts {
    pub daily: DSData,
    pub hourly: DSData,
    pub currently: DSCurrent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DSData {
    pub summary: String,
    pub data: Vec<DSForecast>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DSForecast {
    pub time: i64,
    pub icon: String,
    pub precip_probability: Option<f32>,
    pub temperature_high: Option<f32>,
    pub temperature_low: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DSCurrent {
    pub temperature: f32,
    pub icon: String,
    pub precip_probability: Option<f32>,
}

impl DarkSkyRestClient {
    pub fn new(token: String, location: LatLong) -> DarkSkyRestClient {
        DarkSkyRestClient {
            token,
            location,
        }
    }

    pub fn get_client(&self) -> Result<RestClient, Error> {
        let mut client = RestClient::new(URL_BASE)?;
        Ok(client)
    }
}

impl DarkSkyClient for DarkSkyRestClient {
    fn forecasts(&mut self) -> Result<DSForecasts, Error> {
        let mut client = self.get_client()?;
        let args = DSArgs{ token: self.token.clone(), location: self.location.clone() };
        client.get::<DSArgs, DSForecasts>(args)
    }
}

impl RestPath<DSArgs> for DSForecasts {
    fn get_path(ds: DSArgs) -> Result<String, Error> { Ok(format!("forecast/{}/{}", ds.token, ds.location.to_string())) }
}