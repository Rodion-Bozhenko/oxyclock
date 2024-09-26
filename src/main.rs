use std::time::Duration;

use iced::{
    theme,
    widget::{button, column, container, row, text, text_input, TextInput},
    Alignment, Background, Border, Color, Element, Font, Length, Subscription, Task, Theme,
};

fn main() -> iced::Result {
    iced::application("Oxyclock", Timer::update, Timer::view)
        .theme(Timer::theme)
        .subscription(Timer::subscription)
        .font(include_bytes!("../fonts/oxyclock-fonts.ttf").as_slice())
        .run()
}

#[derive(Debug, Clone)]
enum Msg {
    Tick,
    Start,
    Stop,
    Reset,
    Hours(String),
    Minutes(String),
    Seconds(String),
}

#[derive(Debug, Clone, PartialEq)]
enum State {
    Running,
    Stopped,
}

struct Timer {
    time: Duration,
    elapsed: Duration,
    state: State,
    hours: String,
    minutes: String,
    seconds: String,
}

impl Timer {
    fn new() -> Self {
        Self {
            time: Duration::from_secs(0),
            elapsed: Duration::from_secs(0),
            state: State::Stopped,
            hours: String::from("00"),
            minutes: String::from("00"),
            seconds: String::from("00"),
        }
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Start => {
                let duration = get_duration(&self.hours, &self.minutes, &self.seconds);
                if let Ok(duration) = duration {
                    self.state = State::Running;
                    self.time = duration;
                    self.elapsed = Duration::from_secs(0);
                }
                Task::none()
            }
            Msg::Stop => {
                self.state = State::Stopped;
                self.update_elapsed_hms();
                Task::none()
            }
            Msg::Reset => {
                self.time = Duration::from_secs(0);
                self.update_elapsed_hms();
                Task::none()
            }
            Msg::Tick => {
                if self.time <= Duration::from_secs(1) {
                    self.state = State::Stopped;
                    return Task::none();
                }
                let tick = Duration::from_secs(1);
                self.time -= tick;
                self.elapsed += tick;
                Task::none()
            }
            Msg::Hours(hours) => {
                self.hours = hours;
                Task::none()
            }
            Msg::Minutes(minutes) => {
                self.minutes = minutes;
                Task::none()
            }
            Msg::Seconds(seconds) => {
                self.seconds = seconds;
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Msg> {
        match self.state {
            State::Running => iced::time::every(Duration::from_secs(1)).map(|_| Msg::Tick),
            State::Stopped => Subscription::none(),
        }
    }

    fn view(&self) -> Element<Msg> {
        let started = self.state == State::Running;

        let start_button = if started {
            container(button(stop_icon()).on_press(Msg::Stop))
        } else {
            container(
                row![
                    button(reset_icon()).on_press(Msg::Reset),
                    button(start_icon()).on_press(Msg::Start),
                ]
                .spacing(10),
            )
        };

        let time_container = if started {
            container(text(time_to_hms_string(self.time)).size(50))
        } else {
            let time_inputs = row![
                time_input(&self.hours, Msg::Hours),
                time_input(&self.minutes, Msg::Minutes),
                time_input(&self.seconds, Msg::Seconds),
            ];
            container(time_inputs)
        };

        let column = column![time_container, start_button];

        container(column.spacing(10).align_x(Alignment::Center))
            .center(Length::Fill)
            .into()
    }

    fn theme(&self) -> theme::Theme {
        theme::Theme::Nord
    }

    fn update_elapsed_hms(&mut self) {
        let mut elapsed = self.time.as_secs();
        self.hours = format!("{:02}", (elapsed / 3600));
        elapsed %= 3600;
        self.minutes = format!("{:02}", (elapsed / 60));
        elapsed %= 60;
        self.seconds = format!("{:02}", elapsed);
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

fn time_input<F>(value: &str, msg: F) -> TextInput<'static, Msg, Theme, iced::Renderer>
where
    F: 'static + Fn(String) -> Msg,
{
    // text_input("", &format!("{:02}", value.parse::<u64>().unwrap()))
    text_input("", value)
        .width(70)
        .size(50)
        .style(|_, _| text_input::Style {
            border: Border {
                width: 0.0,
                ..Border::default()
            },
            background: Background::Color(Color::default()),
            icon: Color::WHITE,
            placeholder: Color::WHITE,
            value: Color::WHITE,
            selection: Color::default(),
        })
        .on_input(msg)
}

fn get_duration(
    hours: &str,
    minutes: &str,
    seconds: &str,
) -> Result<Duration, Box<dyn std::error::Error>> {
    let hours_to_secs = hours.parse::<u64>()? * 3600;
    let minutes_to_secs = minutes.parse::<u64>()? * 60;
    let seconds = seconds.parse::<u64>()?;
    let total_secs = hours_to_secs + minutes_to_secs + seconds;

    Ok(Duration::from_secs(total_secs))
}

fn time_to_hms_string(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn start_icon<'a>() -> Element<'a, Msg> {
    icon('\u{e801}')
}

fn stop_icon<'a>() -> Element<'a, Msg> {
    icon('\u{e802}')
}

fn reset_icon<'a>() -> Element<'a, Msg> {
    icon('\u{e800}')
}

fn icon<'a>(codepoint: char) -> Element<'a, Msg> {
    const ICON_FONT: Font = Font::with_name("oxyclock-fonts");
    text(codepoint).font(ICON_FONT).into()
}
