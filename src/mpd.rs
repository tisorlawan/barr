use crate::{Widget, WidgetOutput};
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Mutex;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Eq, PartialEq)]
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
    stream: Mutex<Result<TcpStream, MPDError>>,
    interval: Duration,
    pause_color: String,
}

#[derive(Debug, Copy, Clone)]
pub enum MPDError {
    ConnectionError,
}

impl From<std::io::Error> for MPDError {
    fn from(_: std::io::Error) -> Self {
        MPDError::ConnectionError
    }
}

impl From<&MPDError> for MPDError {
    fn from(e: &MPDError) -> Self {
        e.clone()
    }
}

#[async_trait]
impl Widget for MPD {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self, _pos: usize) -> WidgetOutput {
        let mut song = self.current_song().await;
        if song.is_err() {
            if !self.reconnect().await {
                return WidgetOutput {
                    text: "<span foreground='red'>No MPD</span>".to_string(),
                    use_default_fg: false,
                    use_default_bg: true,
                };
            } else {
                song = self.current_song().await;
            }
        }

        let mut status = self.status().await;
        if status.is_err() {
            if !self.reconnect().await {
                return WidgetOutput {
                    text: "<span foreground='red'>No MPD</span>".to_string(),
                    use_default_fg: false,
                    use_default_bg: true,
                };
            } else {
                status = self.status().await;
            }
        }

        if song.is_ok() && status.is_ok() {
            let song = song.unwrap();
            let status = status.unwrap();
            let mut output = format!("[{}] {} - {}", status.percentage, song.artist, song.title);

            let (mut use_default_fg, use_default_bg) = (true, true);
            match status.state {
                State::Pause => {
                    use_default_fg = false;
                    output = format!(
                        "<span foreground='{}'><i>{}</i></span>",
                        self.pause_color, output
                    )
                }
                State::Stop => {
                    output = format!(
                        "<span foreground='{}'><i>/{} - {}/</i></span>",
                        self.pause_color, song.artist, song.title
                    );
                }
                State::Play => (),
            }

            WidgetOutput {
                text: output,
                use_default_fg,
                use_default_bg,
            }
        } else {
            WidgetOutput {
                text: "<span foreground='red'>No MPD</span>".to_string(),
                use_default_fg: false,
                use_default_bg: true,
            }
        }
    }
}

impl MPD {
    pub async fn new(interval: Duration) -> Self {
        let pause_color = "grey".to_string();

        let stream = match TcpStream::connect("localhost:6600").await {
            Ok(mut stream) => {
                let mut buf = vec![0u8; 1024];
                stream.read(&mut buf).await.unwrap();
                Mutex::new(Ok(stream))
            }
            Err(_) => Mutex::new(Err(MPDError::ConnectionError)),
        };

        Self {
            interval,
            stream: stream,
            pause_color,
        }
    }

    pub async fn reconnect(&self) -> bool {
        if let Ok(mut stream) = TcpStream::connect("localhost:6600").await {
            let mut buf = vec![0u8; 1024];
            stream.read(&mut buf).await.unwrap();

            let mut s = self.stream.lock().await;
            *s = Ok(stream);
            true
        } else {
            false
        }
    }

    async fn current_song(&self) -> Result<Song, MPDError> {
        let mut stream = self.stream.lock().await;

        stream.as_ref()?.write_all(b"currentsong\n").await?;

        let mut buf = [0; 1024];
        stream.as_ref()?.read(&mut buf).await?;

        let s: Vec<String> = std::str::from_utf8(&buf)
            .unwrap()
            .trim_matches(char::from(0))
            .lines()
            .filter(|l| *l != "OK")
            .filter(|l| l.starts_with("Artist") || l.starts_with("Title"))
            .map(Into::into)
            .collect();

        if s.is_empty() {
            *stream = Err(MPDError::ConnectionError);
            Err(MPDError::ConnectionError)
        } else {
            Ok(Song {
                artist: s[0][8..].to_owned(),
                title: s[1][7..].to_owned(),
            })
        }
    }

    async fn status(&self) -> Result<Status, MPDError> {
        let stream = self.stream.lock().await;
        stream.as_ref()?.write_all(b"status\n").await?;

        let mut buf = [0u8; 1024];
        stream.as_ref()?.read(&mut buf).await?;

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

        Ok(Status {
            consume: s.get("consume").unwrap() == "1",
            single: s.get("single").unwrap() == "1",
            random: s.get("random").unwrap() == "1",
            repeat: s.get("repeat").unwrap() == "1",
            state,
            percentage: (elapsed * 100f64 / duration).floor() as u8,
        })
    }
}
