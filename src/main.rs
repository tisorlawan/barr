use barr::{Alsa, Barr, Battery, Brightness, Date, Memory, Network, Wifi, CPU, MPD};
use std::time::Duration;

fn main() {
    let mut barr = Barr::new();
    let one_second = Duration::from_secs(1);

    smol::run(async {
        barr.add_widget(Box::new(Alsa::new(one_second)));
        barr.add_widget(Box::new(MPD::new(one_second).await));
        barr.add_widget(Box::new(Brightness::new(one_second)));
        barr.add_widget(Box::new(Memory::new(one_second)));
        barr.add_widget(Box::new(CPU::new(one_second)));
        barr.add_widget(Box::new(Wifi::new(one_second)));
        barr.add_widget(Box::new(Network::new(one_second, "wlp2s0")));
        barr.add_widget(Box::new(Battery::new(one_second)));
        barr.add_widget(Box::new(Date::new(one_second)));

        barr.run().await;
    });
}
