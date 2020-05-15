pub mod date;
pub mod network;

pub mod battery;
pub mod brightness;
pub mod cpu;
pub mod memory;
pub mod wifi;

pub mod alsa;
pub mod mpd;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum WidgetTag {
    Date,
    NetworkSpeed,
    Battery,
    Wifi,
    Memory,
    Cpu,
    Brightness,
    Alsa,
    Mpd,
}

#[derive(Debug)]
pub struct Output {
    pub tag: WidgetTag,
    pub text: String,
}
