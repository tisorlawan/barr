use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use chrono::prelude::*;
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
pub struct Date {
    interval: Duration,
}

#[async_trait]
impl Widget for Date {
    async fn get_output(&self) -> WidgetOutput {
        WidgetOutput {
            text: Self::get_date("%a, %d %b %H:%M:%S"),
            use_default_foreground: true,
            use_default_background: true,
        }
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}

impl Date {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    fn get_date(fmt: &str) -> String {
        let now = Local::now();
        now.format(fmt).to_string()
    }
}
