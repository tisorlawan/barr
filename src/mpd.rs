use crate::{Widget, WidgetOutput};

use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Mutex;

use async_trait::async_trait;

use std::collections::HashMap;
use std::str::{self, Utf8Error};
use std::time::Duration;

#[derive(Debug, Eq, PartialEq)]
enum State {
    Pause,
    Play,
    Stop,
}

#[allow(clippy::struct_excessive_bools)]
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
    ParseError,
    EmptyPlaylist,
}

impl From<std::io::Error> for MPDError {
    fn from(_: std::io::Error) -> Self {
        MPDError::ConnectionError
    }
}

impl From<Utf8Error> for MPDError {
    fn from(_: Utf8Error) -> Self {
        MPDError::ParseError
    }
}

impl From<&MPDError> for MPDError {
    fn from(e: &MPDError) -> Self {
        *e
    }
}

#[async_trait]
impl Widget for MPD {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn get_output(&self) -> WidgetOutput {
        let mut song = self.current_song().await;
        match &song {
            Ok(_) => (),
            Err(MPDError::EmptyPlaylist) => {
                return WidgetOutput {
                    text: "<span foreground='grey'>Empty Playlist</span>".to_string(),
                    use_default_foreground: false,
                    use_default_background: true,
                };
            }
            Err(_) => {
                if self.reconnect().await {
                    song = self.current_song().await;
                } else {
                    return WidgetOutput {
                        text: "<span foreground='red'>No MPD</span>".to_string(),
                        use_default_foreground: false,
                        use_default_background: true,
                    };
                }
            }
        };

        if song.is_err() {
            if self.reconnect().await {
                song = self.current_song().await;
            } else {
            }
        }

        let mut status = self.status().await;
        if status.is_err() {
            if self.reconnect().await {
                status = self.status().await;
            } else {
                return WidgetOutput {
                    text: "<span foreground='red'>No MPD</span>".to_string(),
                    use_default_foreground: false,
                    use_default_background: true,
                };
            }
        }

        if song.is_ok() && status.is_ok() {
            let song = song.unwrap();
            let status = status.unwrap();
            let mut output = format!("[{}] {} - {}", status.percentage, song.artist, song.title);

            let (mut use_default_foreground, use_default_background) = (true, true);
            match status.state {
                State::Pause => {
                    use_default_foreground = false;
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
                use_default_foreground,
                use_default_background,
            }
        } else {
            WidgetOutput {
                text: "<span foreground='red'>No MPD</span>".to_string(),
                use_default_foreground: false,
                use_default_background: true,
            }
        }
    }
}

impl MPD {
    pub async fn new(interval: Duration) -> Self {
        let pause_color = "grey".to_string();

        let stream = match TcpStream::connect("localhost:6600").await {
            Ok(mut stream) => {
                let mut buf = vec![0_u8; 1024];
                stream.read(&mut buf).await.unwrap();
                Mutex::new(Ok(stream))
            }
            Err(_) => Mutex::new(Err(MPDError::ConnectionError)),
        };

        Self {
            interval,
            stream,
            pause_color,
        }
    }

    pub async fn reconnect(&self) -> bool {
        if let Ok(mut stream) = TcpStream::connect("localhost:6600").await {
            let mut buf = vec![0_u8; 1024];
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

        // Bytes should start with "file"
        // Otherwise, it can be assumed that the playlist is empty
        // Most cases if the playlist is empty, it will return "OK"
        if !buf.starts_with(b"file") {
            return Err(MPDError::EmptyPlaylist);
        }

        // Get `artist`, `title`
        let s: Vec<&str> = str::from_utf8(&buf)?
            .trim_matches(char::from(0))
            .lines()
            .filter(|l| l.starts_with("Artist") || l.starts_with("Title"))
            .collect();

        if s.is_empty() {
            *stream = Err(MPDError::ConnectionError);
            Err(MPDError::ConnectionError)
        } else {
            let get_value = |s: &str| -> String {
                s.chars()
                    .skip_while(|s| !s.is_whitespace())
                    .skip(1)
                    .collect::<String>()
            };

            let artist = get_value(s[0]);
            let title = get_value(s[1]);

            Ok(Song { artist, title })
        }
    }

    async fn status(&self) -> Result<Status, MPDError> {
        let stream = self.stream.lock().await;
        stream.as_ref()?.write_all(b"status\n").await?;

        let mut buf = [0_u8; 1024];
        stream.as_ref()?.read(&mut buf).await?;

        let s: HashMap<&str, &str> = str::from_utf8(&buf)
            .unwrap()
            .trim_matches(char::from(0))
            .lines()
            .filter_map(|l| {
                if l == "OK" {
                    None
                } else {
                    let mut l = l.split(": ");
                    Some((l.next().unwrap(), l.next().unwrap()))
                }
            })
            .collect();

        let elapsed: f64 = s.get("elapsed").unwrap_or(&"1").parse().unwrap();
        let duration: f64 = s.get("duration").unwrap_or(&"1").parse().unwrap();

        let state = match s.get("state").unwrap().as_ref() {
            "pause" => State::Pause,
            "stop" => State::Stop,
            _ => State::Play,
        };

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Ok(Status {
            consume: s.get("consume").unwrap() == &"1",
            single: s.get("single").unwrap() == &"1",
            random: s.get("random").unwrap() == &"1",
            repeat: s.get("repeat").unwrap() == &"1",
            state,
            percentage: (elapsed * 100_f64 / duration).floor() as u8,
        })
    }
}
