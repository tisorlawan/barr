use alsa::mixer::{Mixer, SelemChannelId, SelemId};
use async_trait::async_trait;

use crate::{Widget, WidgetOutput};
use std::time::Duration;

pub struct Alsa {
    interval: Duration,
}

#[async_trait]
impl Widget for Alsa {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        WidgetOutput {
            text: self.get_volume(),
        }
    }
}

impl Alsa {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
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
