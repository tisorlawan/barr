use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use battery::State;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

#[derive(Debug)]
pub struct Battery {
    interval: Duration,

    ac_color: String,
}

#[derive(Debug)]
struct BatteryInfo {
    pub state: battery::State,
    pub value: f32,
}

impl Display for BatteryInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {:.0}", self.state, self.value)
    }
}

#[async_trait]
impl Widget for Battery {
    async fn get_output(&self, _pos: usize) -> WidgetOutput {
        let info = Self::battery_stat().unwrap();

        let text = {
            match info.state {
                State::Unknown | State::Full => {
                    format!("<span foreground='{}'><b>︇</b></span>", self.ac_color)
                }
                State::Charging => format!("[C] {:.0}", info.value),
                State::Discharging => format!(" {:.0}", info.value),
                State::Empty | State::__Nonexhaustive => {
                    format!("<span foreground='{}'><b>︇</b></span>", "red")
                }
            }
        };
        WidgetOutput {
            text: text,
            use_default_fg: true,
            use_default_bg: true,
        }
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}

impl Battery {
    pub fn new(interval: Duration) -> Self {
        let ac_color = "cyan".to_string();
        Self { interval, ac_color }
    }

    fn battery_stat() -> Result<BatteryInfo, battery::Error> {
        let battery = battery::Manager::new()?.batteries()?.next().unwrap()?;

        Ok(BatteryInfo {
            state: battery.state(),
            value: battery.state_of_charge().value * 100_f32,
        })
    }
}
