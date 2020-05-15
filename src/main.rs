use async_std::sync::channel;
use smol::Task;
use std::time::Duration;

use barr::alsa::{AlsaWidget, AlsaWidgetConfig};
use barr::battery::{BatteryWidget, BatteryWidgetConfig};
use barr::brightness::{BrightnessWidget, BrightnessWidgetConfig};
use barr::cpu::{CpuWidget, CpuWidgetConfig};
use barr::date::{DateWidget, DateWidgetConfig};
use barr::memory::{RamWidget, RamWidgetConfig};
use barr::mpd::{MpdWidget, MpdWidgetConfig};
use barr::network::{NetworkWidget, NetworkWidgetConfig};
use barr::wifi::{WifiWidget, WifiWidgetConfig};

use barr::Output;

macro_rules! widget_run_detach {
    ($i:expr) => {{
        Task::spawn(async move {
            $i.stream_output().await;
        })
        .detach();
    }};
}

macro_rules! widget_def {
    ($widget:ty, $config:ident, $sender:ident) => {{
        let a = <$widget>::new(
            $config {
                interval: Duration::from_secs(1),
            },
            $sender.clone(),
        );
        a
    }};
}

fn main() {
    let (sender, receiver) = channel::<Output>(100);

    let date = widget_def!(DateWidget, DateWidgetConfig, sender);
    let battery = widget_def!(BatteryWidget, BatteryWidgetConfig, sender);
    let wifi = widget_def!(WifiWidget, WifiWidgetConfig, sender);
    let ram = widget_def!(RamWidget, RamWidgetConfig, sender);
    let mut cpu = widget_def!(CpuWidget, CpuWidgetConfig, sender);
    let brightness = widget_def!(BrightnessWidget, BrightnessWidgetConfig, sender);
    let alsa = widget_def!(AlsaWidget, AlsaWidgetConfig, sender);
    let mpd = widget_def!(MpdWidget, MpdWidgetConfig, sender);

    let network_speed = NetworkWidget::new(
        NetworkWidgetConfig {
            interface: "wlp2s0".to_string(),
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    smol::run(async {
        widget_run_detach!(date);
        widget_run_detach!(network_speed);
        widget_run_detach!(battery);
        widget_run_detach!(wifi);
        widget_run_detach!(ram);
        widget_run_detach!(cpu);
        widget_run_detach!(brightness);
        widget_run_detach!(alsa);
        widget_run_detach!(mpd.await);

        loop {
            let output = receiver.recv().await.unwrap();
            println!("{:?}", output);
        }
    });
}
