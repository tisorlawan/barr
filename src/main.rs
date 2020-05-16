use async_std::sync::channel;
use smol::Task;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use barr::{Output, WidgetTag};

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
        widget_def!($path, $sender, 1)
    }};

    ($path:path, $sender:ident, $dur_sec:expr) => {{
        use $path as base;
        let a = base::Widget::new(
            base::Config {
                interval: Duration::from_secs($dur_sec),
            },
            $sender.clone(),
        );
        a
    }};
}

fn main() {
    let (sender, receiver) = channel::<Output>(100);

    let mut tags: Vec<WidgetTag> = Vec::new();

    smol::run(async {
        let alsa = widget_def!(barr::alsa, sender);
        tags.push(alsa.tag());

        let mut mpd = widget_def!(barr::mpd, sender).await;
        tags.push(mpd.tag());

        let brightness = widget_def!(barr::brightness, sender);
        tags.push(brightness.tag());

        let mut cpu = widget_def!(barr::cpu, sender);
        tags.push(cpu.tag());

        let ram = widget_def!(barr::memory, sender);
        tags.push(ram.tag());

        let wifi = widget_def!(barr::wifi, sender);
        tags.push(wifi.tag());

        let network_speed = barr::network::Widget::new(
            barr::network::Config {
                interface: "wlp2s0".to_string(),
                interval: Duration::from_secs(1),
            },
            sender.clone(),
        );
        tags.push(network_speed.tag());

        let battery = widget_def!(barr::battery, sender);
        tags.push(battery.tag());

        let date = barr::date::Widget::new(
            barr::date::Config::new("%a, %d %b %H:%M:%S", Duration::from_secs(1)),
            sender.clone(),
        );
        tags.push(date.tag());

        let tag_indices: HashMap<WidgetTag, usize> =
            tags.iter().enumerate().map(|(i, v)| (*v, i)).collect();
        let mut outs: Vec<String> = {
            let mut v = vec![];
            for _ in tags {
                v.push("".to_string());
            }
            v
        };

        widget_run_detach!(
            date,
            network_speed,
            battery,
            wifi,
            ram,
            cpu,
            brightness,
            alsa,
            mpd,
        );

        let mut last = Instant::now();
        loop {
            let o = receiver.recv().await.unwrap();
            let now = Instant::now();

            let diff = now - last;

            outs[tag_indices[&o.tag]] = o.text;

            if diff.as_secs() >= 1 {
                last = now;
                println!("{:?}", outs);
            }
        }
    });
}
