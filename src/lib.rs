use async_std::sync::{channel, Receiver, Sender};
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

type Handler = Box<dyn Widget + Send + Sync + 'static>;

pub struct Barr {
    widgets: Vec<Arc<Handler>>,
    sender: Sender<(usize, WidgetOutput)>,
    receiver: Receiver<(usize, WidgetOutput)>,
}

impl Default for Barr {
    fn default() -> Self {
        Self::new()
    }
}

impl Barr {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<(usize, WidgetOutput)>(100);
        Self {
            widgets: vec![],
            sender,
            receiver,
        }
    }

    pub fn add_widget(&mut self, widget: Handler) {
        self.widgets.push(Arc::new(widget));
    }

    pub async fn run(&self) {
        for (i, widget) in self.widgets.iter().enumerate() {
            let widget = widget.clone();
            let sender = self.sender.clone();

            Task::spawn(async move {
                loop {
                    Timer::after(widget.interval()).await;
                    sender.send((i, widget.get_output().await)).await;
                }
            })
            .detach();
        }

        loop {
            let (i, output) = self.receiver.recv().await.unwrap();
            println!("{} -> {:?}", i, output);
        }
    }
}
