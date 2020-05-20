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

    async fn get_output(&self, _pos: usize) -> WidgetOutput {
        match Self::get_volume() {
            Some((vol, is_muted)) => {
                let (mut use_default_fg, use_default_bg) = (true, true);
                let text = if is_muted {
                    use_default_fg = true;
                    format!("<span foreground='red'><i>{} {}</i></span>", self.icon, vol)
                } else {
                    format!("{} {}", self.icon, vol)
                };

                WidgetOutput {
                    text,
                    use_default_fg,
                    use_default_bg,
                }
            }
            None => WidgetOutput {
                text: format!(
                    "<span foreground='red'><i>{}</i></span>",
                    "Can't get volume"
                ),
                use_default_fg: true,
                use_default_bg: true,
            },
        }
    }
}

impl Alsa {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            icon: "".to_string(),
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
