use async_trait::async_trait;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;

use crate::{Widget, WidgetOutput};
use std::time::Duration;

pub struct Wifi {
    interval: Duration,
}

#[async_trait]
impl Widget for Wifi {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        let wifi = Self::get_wifi_ssid();
        if wifi.is_none() {
            WidgetOutput {
                text: "Not Connected".to_string(),
            }
        } else {
            let quality = Self::get_current_wifi_quality().unwrap().abs();
            WidgetOutput {
                text: format!("{} - {:.0}", wifi.unwrap(), quality),
            }
        }
    }
}

impl Wifi {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    pub async fn stream_output(&self) {}

    fn get_current_wifi_quality() -> Option<f64> {
        let file = File::open("/proc/net/wireless").ok()?;
        let reader = BufReader::new(file);

        let quality: String = reader
            .lines()
            .nth(2)?
            .unwrap()
            .split_whitespace()
            .map(Into::into)
            .nth(3)?;

        Some(quality.parse::<f64>().unwrap() * (10.0 / 7.0))
    }

    fn get_wifi_ssid() -> Option<String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg("iwgetid -r")
            .output()
            .expect("Failed to execute `iwgetid`")
            .stdout;
        let output = String::from_utf8(output).unwrap().trim().to_owned();
        if output.is_empty() {
            None
        } else {
            Some(output)
        }
    }
}
