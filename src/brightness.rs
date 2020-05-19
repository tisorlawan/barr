use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use std::time::Duration;

#[derive(Debug)]
pub struct Brightness {
    interval: Duration,
    icon: String,
}

#[async_trait]
impl Widget for Brightness {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self, _pos: usize) -> WidgetOutput {
        WidgetOutput {
            text: format!(
                "{} {:.0}",
                self.icon,
                (self.get_file_content().await / 7500_f64) * 100_f64
            ),
            use_default_fg: true,
            use_default_bg: true,
        }
    }
}

impl Brightness {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            icon: "".to_string(),
        }
    }

    pub async fn get_file_content(&self) -> f64 {
        let s = async_std::fs::read_to_string("/sys/class/backlight/intel_backlight/brightness")
            .await
            .unwrap();
        s.trim().parse().unwrap()
    }
}
