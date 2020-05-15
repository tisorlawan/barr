use async_std::sync::channel;
use smol::Task;
use std::time::Duration;

use barr::Output;

#[allow(unused_macros)]
macro_rules! widget_run_detach {
    ($($i:expr),+$(,)*) => {{
        $(
            Task::spawn(async move {
                $i.stream_output().await;
            })
            .detach();
        )*
    }};
}

macro_rules! widget_def {
    ($path:path, $sender:ident) => {{
        use $path as base;
        let a = base::Widget::new(
            base::Config {
                interval: Duration::from_secs(1),
            },
            $sender.clone(),
        );
        a
    }};
}

fn main() {
    let (sender, receiver) = channel::<Output>(100);

    let network_speed = barr::network::Widget::new(
        barr::network::Config {
            interface: "wlp2s0".to_string(),
            interval: Duration::from_secs(1),
        },
        sender.clone(),
    );

    smol::run(async {
        let date = widget_def!(barr::date, sender);
        let battery = widget_def!(barr::battery, sender);
        let wifi = widget_def!(barr::wifi, sender);
        let ram = widget_def!(barr::memory, sender);
        let mut cpu = widget_def!(barr::cpu, sender);
        let brightness = widget_def!(barr::brightness, sender);
        let alsa = widget_def!(barr::alsa, sender);
        let mpd = widget_def!(barr::mpd, sender);

        widget_run_detach!(
            date,
            network_speed,
            battery,
            wifi,
            ram,
            cpu,
            brightness,
            alsa,
            mpd.await,
        );

        loop {
            let output = receiver.recv().await.unwrap();
            println!("{:?}", output);
        }
    });
}
