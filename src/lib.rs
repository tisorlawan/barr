#![feature(test)]

extern crate test;
pub mod date;
pub mod network;

pub mod battery;
pub mod wifi;

#[derive(Debug, Copy, Clone)]
pub enum WidgetTag {
    Date,
    NetworkSpeed,
    Battery,
    Wifi,
}

#[derive(Debug)]
pub struct Output {
    pub tag: WidgetTag,
    pub text: String,
}
