use crate::{Output, WidgetTag};
use async_std::sync::Sender;
use smol::Timer;
use std::io::Read;
use std::time::Duration;

#[derive(Debug)]
pub struct BrightnessWidgetConfig {
    pub interval: Duration,
}

#[derive(Debug)]
pub struct BrightnessWidget {
    config: BrightnessWidgetConfig,
    sender: Sender<Output>,
    tag: WidgetTag,
}

impl BrightnessWidget {
    pub fn new(config: BrightnessWidgetConfig, sender: Sender<Output>) -> Self {
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
                    text: format!("{:.0}", (self.get_file_content() / 7500_f64) * 100_f64),
                    tag: self.tag,
                })
                .await;
            Timer::after(self.config.interval).await;
        }
    }

    pub fn get_file_content(&self) -> f64 {
        // let mut file =
        //     std::fs::File::open("/sys/class/backlight/intel_backlight/brightness").unwrap();
        // let mut buf = String::new();
        // file.read_to_string(&mut buf).unwrap();
        // buf.trim().to_owned().parse().unwrap()
        10.0
    }
}

#[cfg(test)]
mod brightness_tests {
    use super::*;

    #[test]
    fn test_get_brightness() {
        let (sender, _) = async_std::sync::channel::<crate::Output>(100);
        let b = BrightnessWidget::new(
            BrightnessWidgetConfig {
                interval: std::time::Duration::from_secs(1),
            },
            sender,
        );
        assert!(b.get_file_content() > 1_f64);
    }
}
