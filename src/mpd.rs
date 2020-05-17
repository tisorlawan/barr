use crate::{Widget, WidgetOutput};
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Mutex;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
enum State {
    Pause,
    Play,
    Stop,
}

#[derive(Debug)]
struct Status {
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    percentage: u8,
    state: State,
}

#[derive(Debug)]
struct Song {
    artist: String,
    title: String,
}

#[derive(Debug)]
pub struct MPD {
    stream: Mutex<TcpStream>,
    interval: Duration,
}

#[async_trait]
impl Widget for MPD {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        let song = self.current_song().await;
        let status = self.status().await;
        let output = format!("[{}] {} - {}", status.percentage, song.artist, song.title);

        WidgetOutput { text: output }
    }
}

impl MPD {
    pub async fn new(interval: Duration) -> Self {
        let mut stream = TcpStream::connect("localhost:6600").await.unwrap();

        let mut buf = vec![0u8; 1024];
        stream.read(&mut buf).await.unwrap();

        Self {
            interval,
            stream: Mutex::new(stream),
        }
    }

    pub async fn pause(&self) {
        self.stream
            .lock()
            .await
            .write_all(b"pause\n")
            .await
            .unwrap();
        self.clear().await;
    }

    async fn current_song(&self) -> Song {
        self.stream
            .lock()
            .await
            .write_all(b"currentsong\n")
            .await
            .unwrap();

        let mut buf = [0; 1024];
        self.stream.lock().await.read(&mut buf).await.unwrap();

        let s: Vec<String> = std::str::from_utf8(&buf)
            .unwrap()
            .trim_matches(char::from(0))
            .lines()
            .filter(|l| *l != "OK")
            .filter(|l| l.starts_with("Artist") || l.starts_with("Title"))
            .map(Into::into)
            .collect();

        Song {
            artist: s[0][8..].to_owned(),
            title: s[1][7..].to_owned(),
        }
    }

    async fn status(&self) -> Status {
        self.stream
            .lock()
            .await
            .write_all(b"status\n")
            .await
            .unwrap();

        let mut buf = [0u8; 1024];
        self.stream.lock().await.read(&mut buf).await.unwrap();

        let s: HashMap<String, String> = std::str::from_utf8(&buf)
            .unwrap()
            .trim_matches(char::from(0))
            .lines()
            .filter(|l| *l != "OK")
            .map(|l| l.split(": "))
            .map(|mut l| (l.next().unwrap().to_owned(), l.next().unwrap().to_owned()))
            .collect();

        let elapsed: f64 = s
            .get("elapsed")
            .unwrap_or(&"1".to_string())
            .parse()
            .unwrap();

        let duration: f64 = s
            .get("duration")
            .unwrap_or(&"1".to_string())
            .parse()
            .unwrap();

        let state = match s.get("state").unwrap().as_ref() {
            "pause" => State::Pause,
            "play" => State::Play,
            "stop" => State::Stop,
            _ => State::Play,
        };

        Status {
            consume: s.get("consume").unwrap() == "1",
            single: s.get("single").unwrap() == "1",
            random: s.get("random").unwrap() == "1",
            repeat: s.get("repeat").unwrap() == "1",
            state,
            percentage: (elapsed * 100f64 / duration).floor() as u8,
        }
    }

    async fn clear(&self) {
        let mut buf = [0u8; 1024];
        self.stream.lock().await.read(&mut buf).await.unwrap();
    }
}
