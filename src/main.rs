use barr::{Alsa, Barr, Battery, Brightness, Date, Memory, Network, Wifi, CPU, MPD};
use std::time::Duration;

fn main() {
    let mut barr = Barr::new();

    smol::run(async {
        barr.add_widget(Box::new(Date::new(Duration::from_secs(1))));
        barr.add_widget(Box::new(Battery::new(Duration::from_secs(1))));
        barr.add_widget(Box::new(Network::new(
            Duration::from_secs(1),
            "wlp2s0".to_string(),
        )));
        barr.add_widget(Box::new(Wifi::new(Duration::from_secs(1))));
        barr.add_widget(Box::new(CPU::new(Duration::from_secs(1))));
        barr.add_widget(Box::new(Memory::new(Duration::from_secs(1))));
        barr.add_widget(Box::new(Brightness::new(Duration::from_secs(1))));
        barr.add_widget(Box::new(MPD::new(Duration::from_secs(1)).await));
        barr.add_widget(Box::new(Alsa::new(Duration::from_secs(1))));

        barr.run().await;
    });
}
