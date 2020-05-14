use crate::{Output, WidgetTag};
use async_std::sync::Sender;
use smol::Timer;
use std::time::Duration;
use sysinfo::{System, SystemExt};

pub struct RamWidgetConfig {
    pub interval: Duration,
}

pub struct RamWidget {
    config: RamWidgetConfig,
    sender: Sender<Output>,
    tag: WidgetTag,
    system: System,
}

impl RamWidget {
    pub fn new(config: RamWidgetConfig, sender: Sender<Output>) -> Self {
        Self {
            config,
            sender,
            tag: WidgetTag::Memory,
            system: System::new_all(),
        }
    }

    pub async fn stream_output(&self) {
        loop {
            self.sender
                .send(Output {
                    text: format!("{:.1}", self.get_used_ram_percentage()),
                    tag: self.tag,
                })
                .await;
            Timer::after(self.config.interval).await;
        }
    }

    /// Get used ram in percentage
    pub fn get_used_ram_percentage(&self) -> f64 {
        (self.system.get_used_memory() as f64 / self.system.get_total_memory() as f64) * 100_f64
    }
}
