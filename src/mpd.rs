use crate::{Output, WidgetTag};
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Sender;
use smol::Timer;
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
pub struct Config {
    pub interval: Duration,
}

#[derive(Debug)]
pub struct Widget {
    stream: TcpStream,
    config: Config,
    sender: Sender<Output>,
    tag: WidgetTag,
}

impl Widget {
    pub async fn new(config: Config, sender: Sender<Output>) -> Self {
        let mut stream = TcpStream::connect("localhost:6600").await.unwrap();

        let mut buf = vec![0u8; 1024];
        stream.read(&mut buf).await.unwrap();

        Self {
            config,
            sender,
            stream,
            tag: WidgetTag::Mpd,
        }
    }

    pub async fn stream_output(&mut self) {
        loop {
            let song = self.current_song().await;
            let status = self.status().await;
            let output = format!("[{}] {} - {}", status.percentage, song.artist, song.title);
            self.sender
                .send(Output {
                    text: output,
                    tag: self.tag,
                })
                .await;
            Timer::after(self.config.interval).await;
        }
    }

    pub async fn pause(&mut self) {
        self.stream.write_all(b"pause\n").await.unwrap();
        self.clear().await;
    }

    async fn current_song(&mut self) -> Song {
        self.stream.write_all(b"currentsong\n").await.unwrap();

        let mut buf = [0; 1024];
        self.stream.read(&mut buf).await.unwrap();

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

    async fn status(&mut self) -> Status {
        self.stream.write_all(b"status\n").await.unwrap();

        let mut buf = [0u8; 1024];
        self.stream.read(&mut buf).await.unwrap();

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
            state: state,
            percentage: (elapsed * 100f64 / duration).floor() as u8,
        }
    }

    async fn clear(&mut self) {
        let mut buf = [0u8; 1024];
        self.stream.read(&mut buf).await.unwrap();
    }
}
