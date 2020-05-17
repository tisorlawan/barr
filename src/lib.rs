use async_std::sync::Sender;
use async_trait::async_trait;
use smol::{Task, Timer};

use std::sync::Arc;
use std::time::Duration;

mod alsa;
mod battery;
mod brightness;
mod cpu;
mod date;
mod memory;
mod mpd;
mod network;
mod wifi;

pub use crate::alsa::Alsa;
pub use crate::battery::Battery;
pub use crate::brightness::Brightness;
pub use crate::cpu::CPU;
pub use crate::date::Date;
pub use crate::memory::Memory;
pub use crate::mpd::MPD;
pub use crate::network::Network;
pub use crate::wifi::Wifi;

#[async_trait]
pub trait Widget {
    async fn get_output(&self) -> WidgetOutput;
    fn interval(&self) -> Duration;
}

#[derive(Debug)]
pub struct WidgetOutput {
    pub text: String,
}

pub struct Barr {
    widgets: Vec<Arc<Box<dyn Widget + Send + Sync + 'static>>>,
    sender: Sender<WidgetOutput>,
}

impl Barr {
    pub fn new(sender: Sender<WidgetOutput>) -> Self {
        Self {
            widgets: vec![],
            sender,
        }
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget + Send + Sync + 'static>) {
        self.widgets.push(Arc::new(widget));
    }

    pub async fn run_detach(&self) {
        for widget in self.widgets.iter() {
            let widget = widget.clone();
            let sender = self.sender.clone();

            Task::spawn(async move {
                loop {
                    Timer::after(widget.interval()).await;
                    sender.send(widget.get_output().await).await;
                }
            })
            .detach();
        }
    }
}
