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

pub struct Network<'a> {
    interval: Duration,
    interface: &'a str,

    network_stats: Mutex<NetworkStats>,
    last_called: Mutex<Instant>,

    rx_icon: String,
    tx_icon: String,
}

#[async_trait]
impl Widget for Network<'_> {
    fn interval(&self) -> Duration {
        self.interval
    }

    #[allow(clippy::cast_precision_loss)]
    async fn get_output(&self) -> WidgetOutput {
        let new_network_stat = Self::get_network_stats(self.interface).unwrap();
        let end = Instant::now();

        let diff = end - *self.last_called.lock().unwrap();

        let rx = (new_network_stat.rx_bytes - self.network_stats.lock().unwrap().rx_bytes) as f64
            / diff.as_secs_f64();
        let tx = (new_network_stat.tx_bytes - self.network_stats.lock().unwrap().tx_bytes) as f64
            / diff.as_secs_f64();

        let (rx_kb, rx_mb) = (rx / 1024.0, rx / 1024.0 / 1024.0);
        let (tx_kb, tx_mb) = (tx / 1024.0, tx / 1024.0 / 1024.0);

        let rx = if rx_mb > 1.0 {
            format!(
                "<span foreground='blue'>{} <b>{:.2}</b></span>",
                self.rx_icon, rx_mb
            )
        } else {
            format!("{} {:.0}", self.rx_icon, rx_kb)
        };

        let tx = if tx_mb > 1.0 {
            format!(
                "<span foreground='blue'>{} <b>{:.2}</b></span>",
                self.tx_icon, tx_mb
            )
        } else {
            format!("{} {:.0}", self.tx_icon, tx_kb)
        };

        let text = format!("{}  {}", rx, tx);

        let mut l = self.network_stats.lock().unwrap();
        *l = new_network_stat;

        let mut l = self.last_called.lock().unwrap();
        *l = end;

        WidgetOutput {
            text,
            use_default_foreground: true,
            use_default_background: true,
        }
    }
}

impl<'a> Network<'a> {
    pub fn new(interval: Duration, interface: &'a str) -> Self {
        let rx_icon = "\u{f0ed}".to_string();
        let tx_icon = "\u{f0ee}".to_string();

        Self {
            interval,
            interface,
            network_stats: Mutex::new(Self::get_network_stats(interface).unwrap()),
            last_called: Mutex::new(Instant::now()),
            rx_icon,
            tx_icon,
        }
    }

    fn get_network_stats(inteface: &str) -> io::Result<NetworkStats> {
        let path_root: String = ("/sys/class/net/".to_string() + inteface) + "/statistics/";
        let stats_file = |file: &str| (&path_root).to_string() + file;

        let rx_bytes: u64 = value_from_file(&stats_file("rx_bytes"))?;
        let tx_bytes: u64 = value_from_file(&stats_file("tx_bytes"))?;
        let rx_packets: u64 = value_from_file(&stats_file("rx_packets"))?;
        let tx_packets: u64 = value_from_file(&stats_file("tx_packets"))?;
        let rx_errors: u64 = value_from_file(&stats_file("rx_errors"))?;
        let tx_errors: u64 = value_from_file(&stats_file("tx_errors"))?;

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
        .map_err(|_e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("File: \"{}\" doesn't contain an int value", &path),
            )
        })
}
