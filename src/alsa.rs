use alsa::mixer::{Mixer, SelemChannelId, SelemId};
use async_trait::async_trait;

use crate::{Widget, WidgetOutput};
use std::time::Duration;

pub struct Alsa {
    interval: Duration,
    icon: String,
}

struct AlsaInfo {
    is_muted: bool,
    volume: i64,
}

#[async_trait]
impl Widget for Alsa {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self, _pos: usize) -> WidgetOutput {
        let info = self.get_volume();

        let (mut use_default_fg, use_default_bg) = (true, true);
        let text = if info.is_muted {
            use_default_fg = false;
            format!(
                "<span foreground='red'><i>{} {}</i></span>",
                self.icon, info.volume
            )
        } else {
            format!("{} {}", self.icon, info.volume)
        };

        WidgetOutput {
            text,
            use_default_fg,
            use_default_bg,
        }
    }
}

impl Alsa {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            icon: "".to_string(),
        }
    }

    fn get_volume(&self) -> AlsaInfo {
        let mixer = Mixer::new("hw:0", true).unwrap();
        let selem = mixer.find_selem(&SelemId::new("Master", 0)).unwrap();

        let (min, max) = selem.get_playback_volume_range();
        let vol = selem.get_playback_volume(SelemChannelId::mono()).unwrap();
        let _p = 100 * (vol - min) / (max - min);

        let is_muted = selem.get_playback_switch(SelemChannelId::mono()).unwrap() == 0;
        AlsaInfo {
            is_muted,
            volume: vol,
        }
    }
}
