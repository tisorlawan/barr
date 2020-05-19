use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use std::sync::Mutex;
use std::time::Duration;
use sysinfo::{System, SystemExt};

pub struct Memory {
    interval: Duration,
    system: Mutex<System>,
    tresholds: Vec<(f64, String)>,
    icon: String,
}

#[async_trait]
impl Widget for Memory {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self, _pos: usize) -> WidgetOutput {
        let ram = self.get_used_ram_percentage();
        let mut text = format!("{} {:.0}", self.icon, ram);

        let mut use_default_fg = true;

        let fg = {
            let mut fg = None;
            for (treshold, color) in self.tresholds.iter().rev() {
                if ram >= *treshold {
                    fg = Some(color);
                    break;
                }
            }
            fg
        };

        if let Some(fg) = fg {
            text = format!("<span foreground='{}'>{}</span>", fg, text);
            use_default_fg = false;
        }

        WidgetOutput {
            text,
            use_default_fg,
            use_default_bg: true,
        }
    }
}

impl Memory {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            system: Mutex::new(System::new_all()),
            tresholds: vec![
                (35_f64, "#E9A072".to_string()),
                (50_f64, "#F2665F".to_string()),
                (80_f64, "#FF0000".to_string()),
            ],
            icon: "".to_string(),
        }
    }

    /// Get used ram in percentage
    pub fn get_used_ram_percentage(&self) -> f64 {
        let mut s = self.system.lock().unwrap();
        s.refresh_memory();
        (s.get_used_memory() as f64 / s.get_total_memory() as f64) * 100_f64
    }
}
