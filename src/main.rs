extern crate argparse;
#[macro_use]
extern crate serde_derive;

use std::fs::File;

use argparse::{ArgumentParser, Store, StoreTrue};
use colored::*;

use crate::darksky_client::DarkSkyRestClient;
use crate::darksky_forecaster::DarkSkyForecaster;
use crate::forecast::{Forecaster, LatLong, Weather, WeatherFormatter};
use crate::forecast::Weather::*;
use chrono::{Datelike, Weekday, Timelike};

mod darksky_forecaster;
mod darksky_client;
mod forecast;


fn main() {
    let mut dsf = create_dark_sky_forecaster();
    let fmt = SimpleFormatter::new();

    let mut daily = false;
    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Print the daily or hourly weather forecast");
        ap.refer(&mut daily)
            .add_option(&["-d", "--daily"], StoreTrue,
                        "Print the Daily forecast (default is hourly)");
        ap.parse_args_or_exit();
    }

    if daily {
        print_daily_forecast(&mut dsf, fmt);
    } else {
        print_hourly_forecast(&mut dsf, fmt);
    }
}

fn print_daily_forecast(dsf: &mut DarkSkyForecaster, fmt: SimpleFormatter) {
    match dsf.daily_forecast() {
        Err(e) => panic!("Failed to reach DarkSky: {}", e),
        Ok(forecast) => {
            for f in forecast {
                let dow = f.date.unwrap().weekday().abbr();
                print!("{}{}", dow, fmt.format(f.weather, f.precip_chance));
            }
            println!();
        }
    }
}

trait HasAbbr {
    fn abbr(&self) -> String;
}

impl HasAbbr for Weekday {
    fn abbr(&self) -> String {
        match self {
            Weekday::Mon => "M",
            Weekday::Tue => "T",
            Weekday::Wed => "W",
            Weekday::Thu => "T",
            Weekday::Fri => "F",
            Weekday::Sat => "S",
            Weekday::Sun => "S",
        }.to_string()
    }
}

fn print_hourly_forecast(dsf: &mut DarkSkyForecaster, fmt: SimpleFormatter) {
    print_current_temp(dsf);

    match dsf.hourly_forecast() {
        Err(e) => panic!("Failed to reach DarkSky: {}", e),
        Ok(forecast) => {
            for f in forecast.iter().take(8) {
                print!("{} ", fmt.format(f.weather, f.precip_chance));
            }
            println!();
        }
    }
}

fn print_current_temp(dsf: &mut DarkSkyForecaster) {
    match dsf.current() {
        Err(e) => {} // If we didn't get the current conditions, just skip it
        Ok(current_) => match current_.temp {
            Some(temp) => print!("{} ", format!("{}", temp as i32).white().bold()),
            None => {}
        }
    }
}

fn create_dark_sky_forecaster() -> DarkSkyForecaster {
    let file = File::open("config/darksky.json").unwrap();
    let ds_config: ApiToken = serde_json::from_reader(file).expect("Badly formatted auth token file!");
    let location = LatLong { latitude: 47.698, longitude: -122.379 };
    let dsc = DarkSkyRestClient::new(ds_config.token, location);
    let mut dsf = DarkSkyForecaster::new(Box::new(dsc));
    dsf
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiToken {
    pub token: String
}

struct SimpleFormatter {}

impl SimpleFormatter {
    pub fn new() -> SimpleFormatter { SimpleFormatter {} }
}

impl WeatherFormatter for SimpleFormatter {
    fn format(&self, w: Weather, precip_chance: f32) -> String {
        let s = match w {
            Sunny => "S".yellow().bold(),
            PartSun => "s".yellow(),
            Cloudy => "C".white(),
            Showers | Rain =>
                if precip_chance > 0.25 {
                    "r".cyan().bold()
                } else {
                    "R".cyan().bold()
                },
            Snow => "*".white().bold(),
            Fog => if precip_chance > 0.25 { "f".cyan().bold() } else { "F".white() },
            Unknown => "?".red()
        };

        s.to_string()
    }
}