use battery;

use crate::{Output, WidgetTag};
use async_std::sync::Sender;
use smol::Timer;
use std::time::Duration;

#[derive(Debug)]
pub struct Config {
    pub interval: Duration,
}

#[derive(Debug)]
pub struct Widget {
    config: Config,
    sender: Sender<Output>,
    tag: WidgetTag,
}

#[derive(Debug)]
struct BatteryInfo {
    pub state: battery::State,
    pub value: f32,
}

impl Widget {
    pub fn new(config: Config, sender: Sender<Output>) -> Self {
        Self {
            config,
            sender,
            tag: WidgetTag::Battery,
        }
    }

    fn battery_stat() -> Result<BatteryInfo, battery::Error> {
        let manager = battery::Manager::new()?;

        for maybe_battery in manager.batteries()? {
            let battery = maybe_battery?;
            return Ok(BatteryInfo {
                state: battery.state(),
                value: battery.state_of_charge().value * 100_f32,
            });
        }
        Ok(BatteryInfo {
            state: battery::State::Unknown,
            value: 0_f32,
        })
    }

    pub async fn stream_output(&self) {
        loop {
            let info = Self::battery_stat().unwrap();

            self.sender
                .send(Output {
                    text: format!("{:?}", info),
                    tag: self.tag,
                })
                .await;
            Timer::after(self.config.interval).await;
        }
    }
}
