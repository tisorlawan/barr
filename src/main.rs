use async_std::sync::channel;
use smol::Task;
use std::time::Duration;

use dwm_bar::battery::{BatteryWidget, BatteryWidgetConfig};
use dwm_bar::date::{DateWidget, DateWidgetConfig};
use dwm_bar::network::{NetworkWidget, NetworkWidgetConfig};
use dwm_bar::wifi::{WifiWidget, WifiWidgetConfig};
use dwm_bar::Output;

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

        loop {
            let output = receiver.recv().await.unwrap();
            println!("{:?}", output);
        }
    });
}
