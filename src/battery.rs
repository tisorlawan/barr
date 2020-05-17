use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use std::time::Duration;

#[derive(Debug)]
pub struct Battery {
    interval: Duration,
}

#[derive(Debug)]
struct BatteryInfo {
    pub state: battery::State,
    pub value: f32,
}

#[async_trait]
impl Widget for Battery {
    async fn get_output(&self) -> WidgetOutput {
        let info = Self::battery_stat().unwrap();
        WidgetOutput {
            text: format!("{:?}", info),
        }
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}

impl Battery {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    fn battery_stat() -> Result<BatteryInfo, battery::Error> {
        let battery = battery::Manager::new()?.batteries()?.next().unwrap()?;

        Ok(BatteryInfo {
            state: battery.state(),
            value: battery.state_of_charge().value * 100_f32,
        })
    }
}
