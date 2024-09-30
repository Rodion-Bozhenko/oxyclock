use iced::Subscription;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::{utils, Msg};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Timer {
    pub id: Uuid,
    pub name: String,
    pub time: Duration,
    pub elapsed: Duration,
    pub state: State,
    pub hours: String,
    pub minutes: String,
    pub seconds: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub enum State {
    Running,
    NotificationSound,
    Stopped,
}

impl Timer {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            name: "".to_string(),
            time: Duration::from_secs(0),
            elapsed: Duration::from_secs(0),
            state: State::Stopped,
            hours: String::from("00"),
            minutes: String::from("00"),
            seconds: String::from("00"),
        }
    }

    pub fn update_elapsed_hms(&mut self) {
        let mut elapsed = self.time.as_secs();
        self.hours = format!("{:02}", (elapsed / 3600));
        elapsed %= 3600;
        self.minutes = format!("{:02}", (elapsed / 60));
        elapsed %= 60;
        self.seconds = format!("{:02}", elapsed);
    }

    pub fn get_duration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let hours_to_secs = self.hours.parse::<u64>()? * 3600;
        let minutes_to_secs = self.minutes.parse::<u64>()? * 60;
        let seconds = self.seconds.parse::<u64>()?;
        let total_secs = hours_to_secs + minutes_to_secs + seconds;

        Ok(Duration::from_secs(total_secs))
    }

    pub fn time_to_hms_string(&self) -> (String, String, String) {
        let total_secs = self.time.as_secs();
        let hours = format!("{:02}", total_secs / 3600);
        let minutes = format!("{:02}", (total_secs % 3600) / 60);
        let seconds = format!("{:02}", total_secs % 60);

        (hours, minutes, seconds)
    }

    pub fn subscription(&self) -> Subscription<Msg> {
        println!("SUBSCRIPTION. STATE: {:?}", self.state);
        match self.state {
            State::Running => iced::time::every(Duration::from_secs(1))
                .with(self.id)
                .map(|s| Msg::Tick(s.0)),
            State::NotificationSound => {
                std::thread::spawn(|| {
                    if let Err(err) = utils::play_notification_sound() {
                        eprintln!("failed to play notification sound: {err}");
                    }
                });
                Subscription::none()
            }
            State::Stopped => Subscription::none(),
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(uuid::Uuid::new_v4())
    }
}
