use alsa::mixer::{Mixer, SelemChannelId, SelemId};

use crate::{Output, WidgetTag};
use async_std::sync::Sender;
use smol::Timer;
use std::time::Duration;

pub struct Config {
    pub interval: Duration,
}

pub struct Widget {
    config: Config,
    sender: Sender<Output>,
    tag: WidgetTag,
}

impl Widget {
    pub fn new(config: Config, sender: Sender<Output>) -> Self {
        Self {
            config,
            sender,
            tag: WidgetTag::Alsa,
        }
    }

    pub fn tag(&self) -> WidgetTag {
        self.tag
    }

    pub async fn stream_output(&self) {
        loop {
            self.sender
                .send(Output {
                    text: self.get_volume(),
                    tag: self.tag,
                })
                .await;
            Timer::after(self.config.interval).await;
        }
    }

    fn get_volume(&self) -> String {
        let mixer = Mixer::new("hw:0", true).unwrap();
        let selem = mixer.find_selem(&SelemId::new("Master", 0)).unwrap();

        let (min, max) = selem.get_playback_volume_range();
        let vol = selem.get_playback_volume(SelemChannelId::mono()).unwrap();
        let p = 100 * (vol - min) / (max - min);

        let is_muted = selem.get_playback_switch(SelemChannelId::mono()).unwrap() == 0;
        format!("{} - {}", is_muted, p)
    }
}
