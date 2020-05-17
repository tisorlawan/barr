use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use std::sync::Mutex;
use std::time::Duration;
use sysinfo::{System, SystemExt};

pub struct Memory {
    interval: Duration,
    system: Mutex<System>,
}

#[async_trait]
impl Widget for Memory {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        WidgetOutput {
            text: format!("{:.1}", self.get_used_ram_percentage()),
        }
    }
}

impl Memory {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            system: Mutex::new(System::new_all()),
        }
    }

    /// Get used ram in percentage
    pub fn get_used_ram_percentage(&self) -> f64 {
        let mut s = self.system.lock().unwrap();
        s.refresh_memory();
        (s.get_used_memory() as f64 / s.get_total_memory() as f64) * 100_f64
    }
}
