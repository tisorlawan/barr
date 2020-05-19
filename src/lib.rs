use async_std::sync::{channel, Receiver, Sender};
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
    async fn get_output(&self, pos: usize) -> WidgetOutput;
    fn interval(&self) -> Duration;
}

#[derive(Debug)]
pub struct WidgetOutput {
    pub text: String,
    pub use_default_bg: bool,
    pub use_default_fg: bool,
}

type Handler = Box<dyn Widget + Send + Sync + 'static>;

pub struct Barr {
    widgets: Vec<Arc<Handler>>,
    sender: Sender<(usize, WidgetOutput)>,
    receiver: Receiver<(usize, WidgetOutput)>,
    outs: Vec<String>,
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
            outs: vec![],
        }
    }

    pub fn add_widget(&mut self, widget: Handler) {
        self.widgets.push(Arc::new(widget));
        self.outs.push(String::new());
    }

    pub async fn run(&mut self) {
        let black = "#0F1419";
        let white = "white";
        let sep = "";

        for (i, widget) in self.widgets.iter().enumerate() {
            let widget = widget.clone();
            let sender = self.sender.clone();

            Task::spawn(async move {
                loop {
                    Timer::after(widget.interval()).await;

                    let mut out = widget.get_output(i).await;

                    let (fg, bg) = if i % 2 == 0 {
                        (black, white)
                    } else {
                        (white, black)
                    };

                    if out.use_default_fg && out.use_default_bg {
                        out.text = format!(
                            "<span background='{}' foreground='{}'> {} </span>",
                            bg, fg, out.text
                        );
                    } else if out.use_default_fg {
                        out.text = format!("<span foreground='{}'> {} </span>", fg, out.text);
                    } else if out.use_default_bg {
                        out.text = format!("<span background='{}'> {} </span>", bg, out.text);
                    }
                    if !(i == 0 && fg == white) {
                        out.text = format!(
                            "<span background='{}' foreground='{}'>{}</span>{}",
                            bg, fg, sep, out.text
                        );
                    }

                    sender.send((i, out)).await;
                }
            })
            .detach();
        }

        loop {
            let (i, output) = self.receiver.recv().await.unwrap();
            self.outs[i] = output.text;

            let cmd = format!("xsetroot -name \"{}\"", self.outs.join(""));
            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .expect("failed to execute process");
        }
    }
}
