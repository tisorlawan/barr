use crate::{Output, WidgetTag};
use async_std::sync::Sender;
use chrono::prelude::*;
use smol::Timer;
use std::time::Duration;

pub struct Config<T>
where
    T: AsRef<str>,
{
    format: T,
    interval: Duration,
}

impl<T> Config<T>
where
    T: AsRef<str>,
{
    pub fn new(format: T, interval: Duration) -> Self {
        Self { format, interval }
    }
}

pub struct Widget<T>
where
    T: AsRef<str>,
{
    config: Config<T>,
    sender: Sender<Output>,
    tag: WidgetTag,
}

impl<T> Widget<T>
where
    T: AsRef<str>,
{
    pub fn new(config: Config<T>, sender: Sender<Output>) -> Self {
        Self {
            config,
            sender,
            tag: WidgetTag::Date,
        }
    }

    pub fn tag(&self) -> WidgetTag {
        self.tag
    }

    pub async fn stream_output(&self) {
        loop {
            let text = Local::now().format(self.config.format.as_ref()).to_string();

            self.sender
                .send(Output {
                    text,
                    tag: self.tag,
                })
                .await;
            Timer::after(self.config.interval).await;
        }
    }
}
