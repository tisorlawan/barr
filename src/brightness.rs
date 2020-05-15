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
    pub tag: WidgetTag,
}

impl Widget {
    pub fn new(config: Config, sender: Sender<Output>) -> Self {
        Self {
            config,
            sender,
            tag: WidgetTag::Brightness,
        }
    }

    pub async fn stream_output(&self) {
        loop {
            self.sender
                .send(Output {
                    text: format!(
                        "{:.0}",
                        (self.get_file_content().await / 7500_f64) * 100_f64
                    ),
                    tag: self.tag,
                })
                .await;
            Timer::after(self.config.interval).await;
        }
    }

    pub async fn get_file_content(&self) -> f64 {
        let s = async_std::fs::read_to_string("/sys/class/backlight/intel_backlight/brightness")
            .await
            .unwrap();
        s.trim().parse().unwrap()
    }
}

#[cfg(test)]
mod brightness_tests {
    use super::*;

    #[async_std::test]
    async fn test_get_brightness() {
        let (sender, _) = async_std::sync::channel::<crate::Output>(100);
        let b = Widget::new(
            Config {
                interval: std::time::Duration::from_secs(1),
            },
            sender,
        );
        assert!(b.get_file_content().await > 1_f64);
    }
}
