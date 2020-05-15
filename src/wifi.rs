use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;

use crate::{Output, WidgetTag};
use async_std::sync::Sender;
use smol::Timer;
use std::time::Duration;

pub struct Config {
    pub interval: Duration,
}

pub struct Widget {
    config: Config,
    sender: Sender<Output>,
    pub tag: WidgetTag,
}

impl Widget {
    pub fn new(config: Config, sender: Sender<Output>) -> Self {
        Self {
            config,
            sender,
            tag: WidgetTag::Wifi,
        }
    }

    pub async fn stream_output(&self) {
        loop {
            let wifi = Self::get_wifi_ssid();
            if wifi.is_none() {
                self.sender
                    .send(Output {
                        text: format!("Not Connected"),
                        tag: self.tag,
                    })
                    .await;
            } else {
                let quality = Self::get_current_wifi_quality().unwrap().abs();
                self.sender
                    .send(Output {
                        text: format!("{} - {:.0}", wifi.unwrap(), quality),
                        tag: self.tag,
                    })
                    .await;
            }
            Timer::after(self.config.interval).await;
        }
    }

    fn get_current_wifi_quality() -> Option<f64> {
        let file = File::open("/proc/net/wireless").ok()?;
        let reader = BufReader::new(file);

        let quality: String = reader
            .lines()
            .skip(2)
            .nth(0)?
            .unwrap()
            .split_whitespace()
            .map(Into::into)
            .nth(3)
            .unwrap();

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

#[cfg(test)]
mod wifi_tests {
    use super::*;
    #[test]

    fn test_wifi_quality() {
        Widget::get_current_wifi_quality();
    }

    #[test]
    fn test_wifi_ssid() {
        assert_eq!(Some("dargombes".to_owned()), Widget::get_wifi_ssid());
    }
}
