use async_std::sync::channel;
use smol::Task;
use std::time::Duration;

use barr::battery::{BatteryWidget, BatteryWidgetConfig};
use barr::date::{DateWidget, DateWidgetConfig};
use barr::network::{NetworkWidget, NetworkWidgetConfig};
use barr::wifi::{WifiWidget, WifiWidgetConfig};
use barr::Output;

fn main() {
    let (sender, receiver) = channel::<Output>(100);

    let date = DateWidget::new(
        DateWidgetConfig {
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    let network_speed = NetworkWidget::new(
        NetworkWidgetConfig {
            interface: "wlp2s0".to_string(),
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    let battery = BatteryWidget::new(
        BatteryWidgetConfig {
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    let wifi = WifiWidget::new(
        WifiWidgetConfig {
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    let ram = barr::memory::RamWidget::new(
        barr::memory::RamWidgetConfig {
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    let mut cpu = barr::cpu::CpuWidget::new(
        barr::cpu::CpuWidgetConfig {
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    let brightness = barr::brightness::BrightnessWidget::new(
        barr::brightness::BrightnessWidgetConfig {
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    let alsa = barr::alsa::AlsaWidget::new(
        barr::alsa::AlsaWidgetConfig {
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    smol::run(async {
        Task::spawn(async move {
            date.stream_output().await;
        })
        .detach();

        Task::spawn(async move {
            network_speed.stream_output().await;
        })
        .detach();

        Task::spawn(async move {
            battery.stream_output().await;
        })
        .detach();

        Task::spawn(async move {
            wifi.stream_output().await;
        })
        .detach();

        Task::spawn(async move {
            ram.stream_output().await;
        })
        .detach();

        Task::spawn(async move {
            cpu.stream_output().await;
        })
        .detach();

        Task::spawn(async move {
            brightness.stream_output().await;
        })
        .detach();

        Task::spawn(async move {
            alsa.stream_output().await;
        })
        .detach();

        loop {
            let output = receiver.recv().await.unwrap();
            println!("{:?}", output);
        }
    });
}
