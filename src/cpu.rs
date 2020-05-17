use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use psutil::cpu::CpuPercentCollector;
use std::sync::Mutex;
use std::time::Duration;

pub struct CPU {
    interval: Duration,
    collector: Mutex<CpuPercentCollector>,
}

#[async_trait]
impl Widget for CPU {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        WidgetOutput {
            text: format!(
                "{:.1}",
                self.collector.lock().unwrap().cpu_percent().unwrap()
            ),
        }
    }
}

impl CPU {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            collector: Mutex::new(CpuPercentCollector::new().unwrap()),
        }
    }
}
