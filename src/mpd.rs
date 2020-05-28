use crate::{Widget, WidgetOutput};

use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Mutex;

use async_trait::async_trait;

use bitflags::bitflags;

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::str::{self, Utf8Error};
use std::time::Duration;

#[derive(Debug, Eq, PartialEq)]
enum State {
    Pause,
    Play,
    Stop,
}

bitflags! {
    struct StatusFlags: u8 {
        const EMPTY = 0b0000;

        const REPEAT = 0b0001;
        const RANDOM = 0b0010;
        const SINGLE = 0b0100;
        const CONSUM = 0b1000;
    }
}

impl Default for StatusFlags {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Display for StatusFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let random = {
            if self.contains(Self::RANDOM) {
                "Z"
            } else {
                "z"
            }
        };
        let consum = {
            if self.contains(Self::CONSUM) {
                "C"
            } else {
                "c"
            }
        };

        let repeat = {
            if self.contains(Self::REPEAT) {
                "R"
            } else {
                "r"
            }
        };

        let single = {
            if self.contains(Self::SINGLE) {
                "Y"
            } else {
                "y"
            }
        };

        write!(f, "{}{}{}{}", random, consum, repeat, single)
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
struct Status {
    flags: StatusFlags,
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
            let mut output = format!(
                "[{}] {} - {} [{}]",
                status.percentage, song.artist, song.title, status.flags
            );

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
                        "<span foreground='{}'><i>/{} - {} [{}]/</i></span>",
                        self.pause_color, song.artist, song.title, status.flags
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
            .filter(|l| l.starts_with("Artist") || l.starts_with("Title") || l.starts_with("file"))
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

            let artist = get_value(s.get(1).unwrap_or(&"Artist: <i>No Artist</i>"));
            let title = get_value(s.get(2).unwrap_or(&"Title: <i>No Title</i>"));

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

        let mut flags: StatusFlags = StatusFlags::default();
        if s.get("consume").unwrap() == &"1" {
            flags.insert(StatusFlags::CONSUM);
        }
        if s.get("single").unwrap() == &"1" {
            flags.insert(StatusFlags::SINGLE);
        }
        if s.get("random").unwrap() == &"1" {
            flags.insert(StatusFlags::RANDOM);
        }
        if s.get("repeat").unwrap() == &"1" {
            flags.insert(StatusFlags::REPEAT);
        }

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Ok(Status {
            flags,
            state,
            percentage: (elapsed * 100_f64 / duration).floor() as u8,
        })
    }
}
