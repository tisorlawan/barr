use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use std::time::Duration;

#[derive(Debug)]
pub struct Brightness {
    interval: Duration,
}

#[async_trait]
impl Widget for Brightness {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        WidgetOutput {
            text: format!(
                "{:.0}",
                (self.get_file_content().await / 7500_f64) * 100_f64
            ),
        }
    }
}

impl Brightness {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    pub async fn get_file_content(&self) -> f64 {
        let s = async_std::fs::read_to_string("/sys/class/backlight/intel_backlight/brightness")
            .await
            .unwrap();
        s.trim().parse().unwrap()
    }
}
