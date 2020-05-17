use crate::{Widget, WidgetOutput};
use async_trait::async_trait;
use std::io;
use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
struct NetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

pub struct Network {
    interval: Duration,
    interface: String,

    tmp_network_stats: Mutex<NetworkStats>,
    tmp_last_called: Mutex<Instant>,
}

#[async_trait]
impl Widget for Network {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        let new_network_stat = Self::get_network_stats(&self.interface).unwrap();
        let end = Instant::now();

        let diff = end - *self.tmp_last_called.lock().unwrap();

        let rx = (new_network_stat.rx_bytes - self.tmp_network_stats.lock().unwrap().rx_bytes)
            as f64
            / diff.as_secs_f64();
        let tx = (new_network_stat.tx_bytes - self.tmp_network_stats.lock().unwrap().tx_bytes)
            as f64
            / diff.as_secs_f64();

        let text = format!(
            "{} {}",
            bytesize::ByteSize::b(rx as u64),
            bytesize::ByteSize::b(tx as u64)
        );

        let mut l = self.tmp_network_stats.lock().unwrap();
        *l = new_network_stat;

        let mut l = self.tmp_last_called.lock().unwrap();
        *l = end;

        WidgetOutput { text }
    }
}

impl Network {
    pub fn new(interval: Duration, interface: String) -> Self {
        let iface = interface.clone();

        Self {
            interval,
            interface,
            tmp_network_stats: Mutex::new(Self::get_network_stats(&iface).unwrap()),
            tmp_last_called: Mutex::new(Instant::now()),
        }
    }

    fn get_network_stats(inteface: &str) -> io::Result<NetworkStats> {
        let path_root: String = ("/sys/class/net/".to_string() + inteface) + "/statistics/";
        let stats_file = |file: &str| (&path_root).to_string() + file;

        let rx_bytes: u64 = value_from_file::<u64>(&stats_file("rx_bytes"))?;
        let tx_bytes: u64 = value_from_file::<u64>(&stats_file("tx_bytes"))?;
        let rx_packets: u64 = value_from_file::<u64>(&stats_file("rx_packets"))?;
        let tx_packets: u64 = value_from_file::<u64>(&stats_file("tx_packets"))?;
        let rx_errors: u64 = value_from_file::<u64>(&stats_file("rx_errors"))?;
        let tx_errors: u64 = value_from_file::<u64>(&stats_file("tx_errors"))?;

        Ok(NetworkStats {
            rx_bytes,
            tx_bytes,
            rx_packets,
            tx_packets,
            rx_errors,
            tx_errors,
        })
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
        .trim_end_matches('\n')
        .parse()
        .or_else(|_| {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("File: \"{}\" doesn't contain an int value", &path),
            ))
        })
}
