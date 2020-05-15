use crate::{Output, WidgetTag};
use async_std::sync::Sender;
use psutil::cpu::CpuPercentCollector;
use smol::Timer;
use std::time::Duration;

pub struct Config {
    pub interval: Duration,
}

pub struct Widget {
    config: Config,
    sender: Sender<Output>,
    tag: WidgetTag,
    collector: CpuPercentCollector,
}

impl Widget {
    pub fn new(config: Config, sender: Sender<Output>) -> Self {
        Self {
            config,
            sender,
            tag: WidgetTag::Cpu,
            collector: CpuPercentCollector::new().unwrap(),
        }
    }

    pub async fn stream_output(&mut self) {
        loop {
            self.sender
                .send(Output {
                    text: format!("{:.1}", self.collector.cpu_percent().unwrap()),
                    tag: self.tag,
                })
                .await;
            Timer::after(self.config.interval).await;
        }
    }
}
