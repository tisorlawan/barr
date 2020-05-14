pub mod date;
pub mod network;

pub mod battery;
pub mod brightness;
pub mod cpu;
pub mod memory;
pub mod wifi;

#[derive(Debug, Copy, Clone)]
pub enum WidgetTag {
    Date,
    NetworkSpeed,
    Battery,
    Wifi,
    Memory,
    Cpu,
    Brightness,
}

#[derive(Debug)]
pub struct Output {
    pub tag: WidgetTag,
    pub text: String,
}
