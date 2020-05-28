use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use psutil::cpu::CpuPercentCollector;
use std::sync::Mutex;
use std::time::Duration;

pub struct CPU {
    interval: Duration,
    collector: Mutex<CpuPercentCollector>,
    tresholds: Vec<(f32, String)>,
    icon: String,
}

#[async_trait]
impl Widget for CPU {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        let cpu = self.collector.lock().unwrap().cpu_percent().unwrap();
        let mut text = format!("{} {:2.0}", self.icon, cpu);

        let mut use_default_fg = true;

        let fg = {
            let mut fg = None;
            for (treshold, color) in self.tresholds.iter().rev() {
                if cpu >= *treshold {
                    fg = Some(color);
                    break;
                }
            }
            fg
        };

        if let Some(fg) = fg {
            text = format!("<span foreground='{}'>{}</span>", fg, text);
            use_default_fg = true;
        }
        WidgetOutput {
            text,
            use_default_foreground: use_default_fg,
            use_default_background: true,
        }
    }
}

impl CPU {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            collector: Mutex::new(CpuPercentCollector::new().unwrap()),
            tresholds: vec![
                (35_f32, "#E9A072".to_string()),
                (50_f32, "#F2665F".to_string()),
                (80_f32, "#FF0000".to_string()),
            ],
            icon: "\u{f0e4}".to_string(),
        }
    }
}
