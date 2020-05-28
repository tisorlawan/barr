#![allow(clippy::used_underscore_binding)]

use async_std::sync::channel;
use async_trait::async_trait;
use smol::{Task, Timer};
use std::process::Command;

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
    pub use_default_background: bool,
    pub use_default_foreground: bool,
}

type Handler = Box<dyn Widget + Send + Sync + 'static>;

pub struct Barr {
    widgets: Vec<Arc<Handler>>,
}

impl Default for Barr {
    fn default() -> Self {
        Self::new()
    }
}

impl Barr {
    pub fn new() -> Self {
        Self { widgets: vec![] }
    }

    pub fn add_widget(&mut self, widget: Handler) {
        self.widgets.push(Arc::new(widget));
    }

    pub async fn run(&mut self) {
        let black = "#0F1419";
        let white = "white";
        let sep = "\u{e0b0}";

        let mut outs: Vec<String> = vec!["".to_owned(); self.widgets.len()];

        let (sender, receiver) = channel::<(usize, WidgetOutput)>(100);
        for (i, widget) in self.widgets.iter().enumerate() {
            let widget = widget.clone();
            let sender = sender.clone();

            Task::spawn(async move {
                loop {
                    let mut out = widget.get_output().await;

                    let (fg, bg) = if i % 2 == 0 {
                        (black, white)
                    } else {
                        (white, black)
                    };

                    if out.use_default_foreground && out.use_default_background {
                        out.text = format!(
                            "<span background='{}' foreground='{}'> {} </span>",
                            bg, fg, out.text
                        );
                    } else if out.use_default_foreground {
                        out.text = format!("<span foreground='{}'> {} </span>", fg, out.text);
                    } else if out.use_default_background {
                        out.text = format!("<span foreground='{}'> {} </span>", bg, out.text);
                    }
                    if !(i == 0 && fg == white) {
                        out.text = format!(
                            "<span background='{}' foreground='{}'>{}</span>{}",
                            bg, fg, sep, out.text
                        );
                    }
                    sender.send((i, out)).await;

                    Timer::after(widget.interval()).await;
                }
            })
            .detach();
        }

        loop {
            let (i, output) = receiver.recv().await.unwrap();
            outs[i] = output.text;

            Command::new("xsetroot")
                .arg("-name")
                .arg(outs.join(""))
                .output()
                .expect("failed to execute process");
        }
    }
}
