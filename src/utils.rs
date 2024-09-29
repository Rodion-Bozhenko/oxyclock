use std::fmt::Display;

pub enum NotificationError {
    PlayError(rodio::PlayError),
    StreamError(rodio::StreamError),
    FsError(std::io::Error),
}

impl Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayError(err) => write!(f, "{err}"),
            Self::StreamError(err) => write!(f, "{err}"),
            Self::FsError(err) => write!(f, "{err}"),
        }
    }
}

pub fn play_notification_sound() -> Result<(), NotificationError> {
    let (_stream, stream_handle) =
        rodio::OutputStream::try_default().map_err(NotificationError::StreamError)?;
    let file = std::io::BufReader::new(
        std::fs::File::open("/usr/share/sounds/lofi-alarm-clock.mp3")
            .map_err(NotificationError::FsError)?,
    );
    let sink = rodio::Sink::try_new(&stream_handle).map_err(NotificationError::PlayError)?;
    let source = rodio::Decoder::new_mp3(file).unwrap();
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
