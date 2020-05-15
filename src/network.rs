use crate::{Output, WidgetTag};
use async_std::sync::Sender;
use smol::Timer;
use std::io;
use std::io::Read;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct NetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

pub struct Config {
    pub interval: Duration,
    pub interface: String,
}

pub struct Widget {
    config: Config,
    sender: Sender<Output>,
    tag: WidgetTag,
}

impl Widget {
    pub fn new(config: Config, sender: Sender<Output>) -> Self {
        Self {
            config,
            sender,
            tag: WidgetTag::NetworkSpeed,
        }
    }

    fn get_network_stats(&self) -> io::Result<NetworkStats> {
        let path_root: String =
            ("/sys/class/net/".to_string() + &self.config.interface) + "/statistics/";
        let stats_file = |file: &str| (&path_root).to_string() + file;

        let rx_bytes: u64 = value_from_file::<u64>(&stats_file("rx_bytes"))?;
        let tx_bytes: u64 = value_from_file::<u64>(&stats_file("tx_bytes"))?;
        let rx_packets: u64 = value_from_file::<u64>(&stats_file("rx_packets"))?;
        let tx_packets: u64 = value_from_file::<u64>(&stats_file("tx_packets"))?;
        let rx_errors: u64 = value_from_file::<u64>(&stats_file("rx_errors"))?;
        let tx_errors: u64 = value_from_file::<u64>(&stats_file("tx_errors"))?;

        Ok(NetworkStats {
            rx_bytes: rx_bytes,
            tx_bytes: tx_bytes,
            rx_packets: rx_packets,
            tx_packets: tx_packets,
            rx_errors: rx_errors,
            tx_errors: tx_errors,
        })
    }

    pub async fn stream_output(&self) {
        let mut network_stat = self.get_network_stats().unwrap();
        let mut start = Instant::now();

        loop {
            Timer::after(self.config.interval).await;

            let new_network_stat = self.get_network_stats().unwrap();
            let end = Instant::now();

            let diff = end - start;

            let rx =
                (new_network_stat.rx_bytes - network_stat.rx_bytes) as f64 / diff.as_secs_f64();
            let tx =
                (new_network_stat.tx_bytes - network_stat.tx_bytes) as f64 / diff.as_secs_f64();

            let text = format!(
                "{} {}",
                bytesize::ByteSize::b(rx as u64),
                bytesize::ByteSize::b(tx as u64)
            );

            self.sender
                .send(Output {
                    text,
                    tag: self.tag,
                })
                .await;

            network_stat = new_network_stat;
            start = end;
        }
    }
}

fn read_file(path: &str) -> io::Result<String> {
    let mut s = String::new();
    std::fs::File::open(path)
        .and_then(|mut f| f.read_to_string(&mut s))
        .map(|_| s)
}

fn value_from_file<T: std::str::FromStr>(path: &str) -> io::Result<T> {
    read_file(path)?
        .trim_end_matches("\n")
        .parse()
        .and_then(|n| Ok(n))
        .or_else(|_| {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("File: \"{}\" doesn't contain an int value", &path),
            ))
        })
}
