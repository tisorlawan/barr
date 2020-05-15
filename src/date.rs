use crate::{Output, WidgetTag};
use async_std::sync::Sender;
use chrono::prelude::*;
use smol::Timer;
use std::time::Duration;

pub struct Config {
    pub interval: Duration,
}

pub struct Widget {
    config: Config,
    sender: Sender<Output>,
    tag: WidgetTag,
}

impl Widget {
    pub fn new(config: Config, sender: Sender<Output>) -> Self {
        Self {
            config,
            sender,
            tag: WidgetTag::Date,
        }
    }

    pub async fn stream_output(&self) {
        loop {
            self.sender
                .send(Output {
                    text: Self::get_date("%a, %d %b %H:%M:%S"),
                    tag: self.tag,
                })
                .await;
            Timer::after(self.config.interval).await;
        }
    }

    fn get_date(fmt: &str) -> String {
        let now = Local::now();
        now.format(fmt).to_string()
    }
}
