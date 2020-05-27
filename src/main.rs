use barr::{Alsa, Barr, Battery, Brightness, Date, Memory, Network, Wifi, CPU, MPD};
use std::time::Duration;

macro_rules! widgets {
    ($($widget:expr),*$(,)?) => {{
        let mut barr = Barr::new();

        smol::run(async {
            $(
                barr.add_widget(Box::new($widget));
            )*

            barr.run().await;
        });
    }};
}

fn main() {
    let sec = Duration::from_secs(1);

    widgets!(
        Alsa::new(sec),
        MPD::new(sec).await,
        Brightness::new(sec),
        Memory::new(sec),
        CPU::new(sec),
        Wifi::new(sec),
        Network::new(sec, "wlp2s0"),
        Battery::new(sec),
        Date::new(sec)
    );
}
