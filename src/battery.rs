#![allow(clippy::non_ascii_literal)]

use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use battery::State;
use notify_rust::{Notification, NotificationUrgency, Timeout};
use std::fmt::{self, Display, Formatter};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Battery {
    interval: Duration,
    tresholds: Vec<(f64, String)>,

    ac_color: String,
    charging_color: String,
    last_notify_critical: Mutex<Option<Instant>>,
    last_notify_full: Mutex<Option<Instant>>,
}

#[derive(Debug)]
struct BatteryInfo {
    state: battery::State,
    value: f32,
}

impl Display for BatteryInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {:.0}", self.state, self.value)
    }
}

#[async_trait]
impl Widget for Battery {
    async fn get_output(&self) -> WidgetOutput {
        match Self::battery_stat() {
            Ok(info) => {
                let mut use_default_fg = true;
                let text = {
                    match info.state {
                        State::Unknown | State::Full => {
                            use_default_fg = false;

                            // Notify when `i == 0` (lowest treshold)
                            let now = Instant::now();
                            let mut last_notify_full = self.last_notify_full.lock().unwrap();

                            if *last_notify_full == None {
                                *last_notify_full = Some(now);
                                Self::notify_full();
                            } else {
                                let diff: Duration = now - (*last_notify_full).unwrap();

                                if diff.as_secs() >= 60 * 15 {
                                    *last_notify_full = Some(now);
                                    Self::notify_full();
                                }
                            }

                            format!("<span foreground='{}'><b>︇</b></span>", self.ac_color)
                        }
                        State::Charging => {
                            // Reset notification immidiately after charged
                            let mut last_notify_critical =
                                self.last_notify_critical.lock().unwrap();
                            *last_notify_critical = None;

                            use_default_fg = false;
                            format!(
                                "<span foreground='{}'>[C] {:.0}</span>",
                                self.charging_color, info.value,
                            )
                        }
                        State::Discharging => {
                            // Reset notification immidiately after charged
                            let mut last_notify_full = self.last_notify_full.lock().unwrap();
                            *last_notify_full = None;

                            let fg = {
                                let mut fg = None;
                                for (i, (treshold, color)) in self.tresholds.iter().enumerate() {
                                    if f64::from(info.value) <= *treshold {
                                        fg = Some(color);

                                        if i > 0 {
                                            break;
                                        }

                                        // Notify when `i == 0` (lowest treshold)
                                        let now = Instant::now();
                                        let mut last_notify =
                                            self.last_notify_critical.lock().unwrap();

                                        if *last_notify == None {
                                            *last_notify = Some(now);
                                            Self::notify_critical();
                                        } else {
                                            let diff: Duration = now - (*last_notify).unwrap();

                                            if diff.as_secs() >= 60 {
                                                *last_notify = Some(now);
                                                Self::notify_critical();
                                            }
                                        }
                                        break;
                                    }
                                }
                                fg
                            };
                            if let Some(fg) = fg {
                                format!("<span foreground='{}'> {:.0}</span>", fg, info.value)
                            } else {
                                format!(" {:.0}", info.value)
                            }
                        }
                        State::Empty | State::__Nonexhaustive => {
                            use_default_fg = false;
                            format!("<span foreground='{}'><b>︇</b></span>", "red")
                        }
                    }
                };
                WidgetOutput {
                    text,
                    use_default_foreground: use_default_fg,
                    use_default_background: true,
                }
            }

            Err(_) => WidgetOutput {
                text: "<span foreground='grey'>No Battery</span>".to_string(),
                use_default_foreground: true,
                use_default_background: true,
            },
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
            tresholds: vec![
                (25_f64, "#FF0000".to_string()),
                (35_f64, "#F2665F".to_string()),
                (50_f64, "#E9A072".to_string()),
            ],
            last_notify_critical: Mutex::new(None),
            last_notify_full: Mutex::new(None),
        }
    }

    fn battery_stat() -> Result<BatteryInfo, battery::Error> {
        let battery = battery::Manager::new()?.batteries()?.next().unwrap()?;

        Ok(BatteryInfo {
            state: battery.state(),
            value: battery.state_of_charge().value * 100_f32,
        })
    }

    fn notify_critical() {
        let res = Notification::new()
            .summary("Battery Critical")
            .body("Please plug the battery")
            .timeout(Timeout::Milliseconds(45 * 1000))
            .hint(notify_rust::NotificationHint::Urgency(
                NotificationUrgency::Critical,
            ))
            .show();

        if res.is_err() {
            eprintln!("Failed create to notification");
        }
    }

    fn notify_full() {
        let res = Notification::new()
            .summary("Battery Fully Charged")
            .timeout(Timeout::Milliseconds(45 * 1000))
            .show();

        if res.is_err() {
            eprintln!("Failed create to notification");
        }
    }
}
