use async_trait::async_trait;

use crate::{Widget, WidgetOutput};
use std::time::Duration;

pub struct Alsa {
    interval: Duration,
    icon: String,
}

#[async_trait]
impl Widget for Alsa {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        let (mut use_default_foreground, use_default_background) = (true, true);

        match Self::get_volume() {
            Some((vol, is_muted)) => {
                let text = if is_muted {
                    use_default_foreground = true;
                    format!("<span foreground='red'><i>{} {}</i></span>", self.icon, vol)
                } else {
                    format!("{} {}", self.icon, vol)
                };

                WidgetOutput {
                    text,
                    use_default_foreground,
                    use_default_background,
                }
            }
            None => WidgetOutput {
                text: format!(
                    "<span foreground='red'><i>{}</i></span>",
                    "Can't get volume"
                ),
                use_default_foreground,
                use_default_background,
            },
        }
    }
}

impl Alsa {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            icon: "\u{f2a0}".to_string(),
        }
    }

    fn get_volume() -> Option<(u8, bool)> {
        let r = std::process::Command::new("sh")
            .arg("-c")
            .arg("amixer sget Master | grep 'Right:'")
            .output()
            .unwrap();

        let x = String::from_utf8(r.stdout).unwrap();
        let vol = x
            .split_whitespace()
            .nth(4)
            .unwrap()
            .chars()
            .skip(1)
            .take_while(|c| *c != '%')
            .collect::<String>()
            .parse::<u8>()
            .unwrap();
        let muted = x.split_whitespace().nth(5) == Some("[off]");

        Some((vol, muted))
    }
}
