use std::{fmt::Display, time::Duration};

use iced::{
    alignment::{Horizontal, Vertical},
    border, theme,
    widget::{
        button, column, container, horizontal_space, row, text, text_input, Button, Container, Row,
        Text, TextInput,
    },
    Alignment, Border, Element, Font, Length, Shadow, Subscription, Task, Theme,
};

fn main() -> iced::Result {
    iced::application("Oxyclock", Timer::update, Timer::view)
        .theme(Timer::theme)
        .subscription(Timer::subscription)
        .font(include_bytes!("../resources/fonts/oxyclock-fonts.ttf").as_slice())
        .run()
}

const TEXT_SIZE: u16 = 50;

#[derive(Debug, Clone)]
enum Msg {
    Tick,
    Start,
    Stop,
    Reset,
    PlayNotification,
    Hours(String),
    Minutes(String),
    Seconds(String),
}

#[derive(Debug, Clone, PartialEq)]
enum State {
    Running,
    NotificationSound,
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
                self.state = State::Stopped;
                self.time = Duration::from_secs(0);
                self.update_elapsed_hms();
                Task::none()
            }
            Msg::PlayNotification => {
                self.state = State::NotificationSound;
                Task::done(Msg::Stop)
            }
            Msg::Tick => {
                if self.state != State::Running {
                    return Task::none();
                }

                if self.time <= Duration::from_secs(1) {
                    if let Err(err) = notify_rust::Notification::new()
                        .summary("Timer is done!")
                        .body("Your timer has finished")
                        .appname("oxyclock")
                        .show()
                    {
                        eprintln!("failed to send notification: {err}");
                    }

                    self.time = Duration::from_secs(0);
                    self.update_elapsed_hms();

                    return Task::done(Msg::PlayNotification);
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
            State::NotificationSound => {
                if let Err(err) = play_notification_sound() {
                    eprintln!("failed to play notification sound: {err}");
                }
                Subscription::none()
            }
            State::Stopped => Subscription::none(),
        }
    }

    fn view(&self) -> Element<Msg> {
        let title = text("Oxyclock").size(70).style(text::primary);

        let started = self.state == State::Running;

        let buttons = if started {
            container(custom_button(stop_icon(), CustomButtonType::Primary).on_press(Msg::Stop))
        } else {
            container(
                row![
                    custom_button(reset_icon(), CustomButtonType::Secondary).on_press(Msg::Reset),
                    custom_button(start_icon(), CustomButtonType::Primary).on_press(Msg::Start),
                ]
                .spacing(10),
            )
        };

        let time_container = if started {
            running_time_container(self.time)
        } else {
            steady_time_container(&self.hours, &self.minutes, &self.seconds)
        };

        let main = column![time_container, buttons]
            .spacing(20)
            .align_x(Alignment::Center);

        container(
            column![
                horizontal_space().height(Length::FillPortion(1)),
                title,
                main,
                horizontal_space().height(Length::FillPortion(2))
            ]
            .align_x(Horizontal::Center)
            .spacing(100),
        )
        .center(Length::Fill)
        .into()
    }

    fn theme(&self) -> theme::Theme {
        theme::Theme::CatppuccinMocha
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
    text_input("", value)
        .align_x(Horizontal::Center)
        .width(70)
        .size(TEXT_SIZE)
        .style(|theme: &Theme, _| {
            let palette = theme.palette();
            text_input::Style {
                background: theme
                    .extended_palette()
                    .secondary
                    .weak
                    .color
                    .scale_alpha(0.1)
                    .into(),
                border: Border::default().rounded(8),
                icon: palette.text,
                placeholder: palette.text.scale_alpha(0.3),
                value: palette.text,
                selection: palette.primary.scale_alpha(0.7),
            }
        })
        .on_input(msg)
}

#[derive(PartialEq)]
enum CustomButtonType {
    Primary,
    Secondary,
}

fn custom_button<'a>(
    content: impl Into<Element<'a, Msg>>,
    button_type: CustomButtonType,
) -> Button<'a, Msg> {
    button(container(content).center(Length::Fill))
        .width(60)
        .height(40)
        .style(move |theme: &Theme, status: button::Status| {
            let palette = theme.palette();
            let ext_palette = theme.extended_palette();
            match status {
                button::Status::Active => button::Style {
                    background: Some(if button_type == CustomButtonType::Primary {
                        ext_palette.primary.strong.color.into()
                    } else {
                        ext_palette.secondary.strong.color.into()
                    }),
                    text_color: palette.text,
                    border: border::rounded(8.0),
                    shadow: Shadow::default(),
                },
                button::Status::Hovered => button::Style {
                    background: Some(palette.primary.into()),
                    text_color: palette.text,
                    border: border::rounded(8.0),
                    shadow: Shadow::default(),
                },
                button::Status::Disabled => button::Style {
                    background: Some(ext_palette.primary.weak.color.into()),
                    text_color: palette.text,
                    border: border::rounded(8.0),
                    shadow: Shadow::default(),
                },
                button::Status::Pressed => button::Style {
                    background: Some(palette.primary.into()),
                    text_color: palette.text,
                    border: border::rounded(8.0),
                    shadow: Shadow::default(),
                },
            }
        })
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

fn time_to_hms_string<'a>(duration: Duration) -> Row<'a, Msg> {
    let total_secs = duration.as_secs();
    let hours = format!("{:02}", total_secs / 3600);
    let minutes = format!("{:02}", (total_secs % 3600) / 60);
    let seconds = format!("{:02}", total_secs % 60);

    fn time_text<'a>(t: String) -> Text<'a> {
        text(t)
            .width(70)
            .size(TEXT_SIZE)
            .align_x(Horizontal::Center)
    }

    row![
        time_text(hours),
        text(":").size(TEXT_SIZE).align_x(Horizontal::Center),
        time_text(minutes),
        text(":").size(TEXT_SIZE).align_x(Horizontal::Center),
        time_text(seconds)
    ]
    .height(70)
    .align_y(Vertical::Center)
}

fn running_time_container<'a>(time: Duration) -> Container<'a, Msg> {
    container(time_to_hms_string(time)).style(|theme: &Theme| container::Style {
        text_color: None,
        background: Some(
            theme
                .extended_palette()
                .secondary
                .weak
                .color
                .scale_alpha(0.1)
                .into(),
        ),
        border: Border::default().rounded(8),
        shadow: Shadow::default(),
    })
}

fn steady_time_container<'a>(hours: &str, minutes: &str, seconds: &str) -> Container<'a, Msg> {
    let time_inputs = row![
        time_input(hours, Msg::Hours),
        text(":").size(TEXT_SIZE),
        time_input(minutes, Msg::Minutes),
        text(":").size(TEXT_SIZE),
        time_input(seconds, Msg::Seconds),
    ]
    .align_y(Vertical::Center)
    .height(70);
    container(time_inputs)
}

enum NotificationError {
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

fn play_notification_sound() -> Result<(), NotificationError> {
    let (_stream, stream_handle) =
        rodio::OutputStream::try_default().map_err(NotificationError::StreamError)?;
    let file = std::io::BufReader::new(
        std::fs::File::open("resources/sounds/lofi-alarm-clock.mp3")
            .map_err(NotificationError::FsError)?,
    );
    let sink = rodio::Sink::try_new(&stream_handle).map_err(NotificationError::PlayError)?;
    let source = rodio::Decoder::new_mp3(file).unwrap();
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
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
