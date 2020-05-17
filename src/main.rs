use async_std::sync::channel;
use barr::{Alsa, Barr, Battery, Brightness, Date, Memory, Network, WidgetOutput, Wifi, CPU, MPD};
use std::time::Duration;

fn main() {
    let (sender, receiver) = channel::<WidgetOutput>(100);

    let mut barr = Barr::new(sender);

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

        barr.run_detach().await;
        loop {
            let o = receiver.recv().await.unwrap();
            println!("{:?}", o);
        }
    });
}
