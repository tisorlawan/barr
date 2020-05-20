use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use battery::State;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

#[derive(Debug)]
pub struct Battery {
    interval: Duration,

    ac_color: String,
    charging_color: String,
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
        match Self::battery_stat() {
            Ok(info) => {
                let mut use_default_fg = true;
                let text = {
                    match info.state {
                        State::Unknown | State::Full => {
                            use_default_fg = false;
                            format!("<span foreground='{}'><b>︇</b></span>", self.ac_color)
                        }
                        State::Charging => {
                            use_default_fg = false;
                            format!(
                                "<span foreground='{}'>[C] {:.0}</span>",
                                self.charging_color, info.value
                            )
                        }
                        State::Discharging => format!(" {:.0}", info.value),
                        State::Empty | State::__Nonexhaustive => {
                            use_default_fg = false;
                            format!("<span foreground='{}'><b>︇</b></span>", "red")
                        }
                    }
                };
                WidgetOutput {
                    text: text,
                    use_default_fg,
                    use_default_bg: true,
                }
            }

            Err(_) => {
                return {
                    WidgetOutput {
                        text: "<span foreground='grey'>No Battery</span>".to_string(),
                        use_default_fg: true,
                        use_default_bg: true,
                    }
                };
            }
        }
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}

impl Battery {
    pub fn new(interval: Duration) -> Self {
        let ac_color = "cyan".to_string();
        let charging_color = "cyan".to_string();
        Self {
            interval,
            ac_color,
            charging_color,
        }
    }

    fn battery_stat() -> Result<BatteryInfo, battery::Error> {
        let battery = battery::Manager::new()?.batteries()?.next().unwrap()?;

        Ok(BatteryInfo {
            state: battery.state(),
            value: battery.state_of_charge().value * 100_f32,
        })
    }
}
