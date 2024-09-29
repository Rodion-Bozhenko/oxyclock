use iced::{
    alignment::{Horizontal, Vertical},
    border, theme,
    widget::{
        button, center, column, container, horizontal_space, row, scrollable, text, text_input,
        Button, Container, Row, Scrollable, Text, TextInput,
    },
    Alignment, Border, Element, Font, Length, Shadow, Subscription, Task, Theme,
};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter, Write};
use std::{fmt::Display, fs::File, time::Duration};
use uuid::Uuid;

mod custom_theme;

fn main() -> iced::Result {
    iced::application("Oxyclock", Oxyclock::update, Oxyclock::view)
        .theme(Oxyclock::theme)
        .subscription(Oxyclock::subscription)
        .font(include_bytes!("../resources/fonts/icons-font.ttf").as_slice())
        .run_with(Oxyclock::load_state)
}

const TEXT_SIZE: u16 = 50;

#[derive(Debug, Clone)]
enum Msg {
    AddTimer,
    DeleteTimer(Uuid),
    Tick(Uuid),
    Start(Uuid),
    Stop(Uuid),
    Reset(Uuid),
    PlayNotification(Uuid),
    Hours(Time),
    Minutes(Time),
    Seconds(Time),
    Name((Uuid, String)),
}

#[derive(Debug, Clone, Hash)]
struct Time {
    id: Uuid,
    time: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
enum State {
    Running,
    NotificationSound,
    Stopped,
}

struct Oxyclock {
    timers: Vec<Timer>,
}

impl Default for Oxyclock {
    fn default() -> Self {
        Oxyclock {
            timers: vec![Timer::default()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct Timer {
    id: Uuid,
    name: String,
    time: Duration,
    elapsed: Duration,
    state: State,
    hours: String,
    minutes: String,
    seconds: String,
}

impl Timer {
    fn new(id: Uuid) -> Self {
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

    fn update_elapsed_hms(&mut self) {
        let mut elapsed = self.time.as_secs();
        self.hours = format!("{:02}", (elapsed / 3600));
        elapsed %= 3600;
        self.minutes = format!("{:02}", (elapsed / 60));
        elapsed %= 60;
        self.seconds = format!("{:02}", elapsed);
    }

    fn subscription(&self) -> Subscription<Msg> {
        match self.state {
            State::Running => iced::time::every(Duration::from_secs(1))
                .with(self.id)
                .map(|s| Msg::Tick(s.0)),
            State::NotificationSound => {
                std::thread::spawn(|| {
                    if let Err(err) = play_notification_sound() {
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

impl Oxyclock {
    fn view(&self) -> Element<Msg> {
        let mut timers = column![].width(Length::Fill).align_x(Horizontal::Center);
        for timer in self.timers.clone() {
            let timer = timer.clone();

            let started = timer.state == State::Running;

            let buttons = if started {
                container(
                    custom_button(pause_icon(), CustomButtonType::Primary, None, None)
                        .on_press(Msg::Stop(timer.id)),
                )
            } else {
                container(
                    row![
                        custom_button(reset_icon(), CustomButtonType::Secondary, None, None)
                            .on_press(Msg::Reset(timer.id)),
                        custom_button(start_icon(), CustomButtonType::Primary, None, None)
                            .on_press(Msg::Start(timer.id)),
                    ]
                    .spacing(10),
                )
            };

            let time_container = if started {
                running_time_container(timer.time)
            } else {
                steady_time_container(
                    timer.id,
                    &timer.name,
                    &timer.hours,
                    &timer.minutes,
                    &timer.seconds,
                )
            };

            let delete_button = container(
                custom_button(
                    delete_icon().size(14f32),
                    CustomButtonType::Secondary,
                    Some(30f32),
                    Some(30f32),
                )
                .on_press(Msg::DeleteTimer(timer.id)),
            )
            .align_right(Length::Fill);

            let timer_container = container(column![
                container(
                    column![
                        delete_button,
                        column![time_container, buttons]
                            .spacing(20)
                            .align_x(Alignment::Center)
                    ]
                    .align_x(Alignment::Center)
                )
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .padding(20)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        text_color: None,
                        background: Some(palette.secondary.base.color.scale_alpha(0.1).into()),
                        border: Border::default().rounded(8),
                        shadow: Shadow::default(),
                    }
                }),
                horizontal_space().height(30).width(Length::Fill)
            ])
            .width(400f32)
            .align_x(Alignment::Center);

            timers = timers.push(timer_container);
        }

        container(center(
            column![
                top_bar(),
                scrollable_content(timers),
                horizontal_space().height(Length::FillPortion(1))
            ]
            .spacing(10),
        ))
        .height(Length::Fill)
        .align_y(Alignment::End)
        .into()
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::AddTimer => {
                self.timers.push(Timer::new(uuid::Uuid::new_v4()));
                self.save_state();
                Task::none()
            }
            Msg::DeleteTimer(id) => {
                let index = self.timers.iter().position(|t| t.id == id).unwrap();
                self.timers.remove(index);
                self.save_state();
                Task::none()
            }
            Msg::Start(id) => {
                let timer = self.timers.iter_mut().find(|x| x.id == id).unwrap();
                let duration = get_duration(&timer.hours, &timer.minutes, &timer.seconds);
                if let Ok(duration) = duration {
                    timer.state = State::Running;
                    timer.time = duration;
                    timer.elapsed = Duration::from_secs(0);
                }
                Task::none()
            }
            Msg::Stop(id) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.state = State::Stopped;
                timer.update_elapsed_hms();
                Task::none()
            }
            Msg::Reset(id) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.state = State::Stopped;
                timer.time = Duration::from_secs(0);
                timer.update_elapsed_hms();
                Task::none()
            }
            Msg::PlayNotification(id) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.state = State::NotificationSound;
                Task::done(Msg::Stop(id))
            }
            Msg::Tick(id) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();

                if timer.state != State::Running {
                    return Task::none();
                }

                if timer.time <= Duration::from_secs(1) {
                    if let Err(err) = notify_rust::Notification::new()
                        .summary("Timer is done!")
                        .body("Your timer has finished")
                        .appname("oxyclock")
                        .show()
                    {
                        eprintln!("failed to send notification: {err}");
                    }

                    timer.time = Duration::from_secs(0);
                    timer.update_elapsed_hms();

                    return Task::done(Msg::PlayNotification(id));
                }

                let tick = Duration::from_secs(1);
                timer.time -= tick;
                timer.elapsed += tick;
                Task::none()
            }
            Msg::Hours(Time { id, time }) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.hours = time;
                Task::none()
            }
            Msg::Minutes(Time { id, time }) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.minutes = time;
                Task::none()
            }
            Msg::Seconds(Time { id, time }) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.seconds = time;
                Task::none()
            }
            Msg::Name((id, name)) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.name = name;
                self.save_state();
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Msg> {
        Subscription::batch(self.timers.iter().map(|t| t.subscription()))
    }

    fn theme(&self) -> theme::Theme {
        custom_theme::arc_dark()
    }

    fn load_state() -> (Oxyclock, Task<Msg>) {
        // Since I don't care about Windows
        #[allow(deprecated)]
        let mut path = std::env::home_dir().unwrap();
        path.push(std::path::Path::new(".local/state/oxyclock/state.json"));
        let state_file = File::open(path).unwrap();
        let reader = BufReader::new(state_file);
        let timers: Vec<Timer> = serde_json::from_reader(reader).unwrap();
        let state = Oxyclock { timers };
        (state, Task::none())
    }

    fn save_state(&self) {
        // Since I don't care about Windows
        #[allow(deprecated)]
        let mut path = std::env::home_dir().unwrap();
        path.push(std::path::Path::new(".local/state/oxyclock/state.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &self.timers).unwrap();
        writer.flush().unwrap();
    }
}

fn time_input<F>(
    timer_id: Uuid,
    value: &str,
    msg: F,
) -> TextInput<'static, Msg, Theme, iced::Renderer>
where
    F: 'static + Fn(Time) -> Msg,
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
                border: Border::default()
                    .rounded(8)
                    .width(1)
                    .color(palette.background.scale_alpha(0.5)),
                icon: palette.text,
                placeholder: palette.text.scale_alpha(0.3),
                value: palette.text,
                selection: palette.primary.scale_alpha(0.7),
            }
        })
        .on_input(move |value| {
            msg(Time {
                id: timer_id,
                time: value,
            })
        })
}

#[derive(PartialEq)]
enum CustomButtonType {
    Primary,
    Secondary,
}

fn custom_button<'a>(
    content: impl Into<Element<'a, Msg>>,
    button_type: CustomButtonType,
    width: Option<f32>,
    height: Option<f32>,
) -> Button<'a, Msg> {
    button(container(content).center(Length::Fill))
        .width(width.unwrap_or(60f32))
        .height(height.unwrap_or(40f32))
        .style(move |theme: &Theme, status: button::Status| {
            let palette = theme.palette();
            let ext_palette = theme.extended_palette();
            match status {
                button::Status::Active => button::Style {
                    background: Some(match button_type {
                        CustomButtonType::Primary => ext_palette.primary.strong.color.into(),
                        CustomButtonType::Secondary => ext_palette.secondary.strong.color.into(),
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

fn steady_time_container<'a>(
    id: Uuid,
    name: &str,
    hours: &str,
    minutes: &str,
    seconds: &str,
) -> Container<'a, Msg> {
    let time_inputs = row![
        time_input(id, hours, Msg::Hours),
        text(":").size(TEXT_SIZE),
        time_input(id, minutes, Msg::Minutes),
        text(":").size(TEXT_SIZE),
        time_input(id, seconds, Msg::Seconds),
    ]
    .align_y(Vertical::Center)
    .height(70);
    container(
        column![
            time_inputs,
            text_input("Name", name)
                .on_input(move |name| Msg::Name((id, name)))
                .width(250f32)
                .padding(8)
                .size(12)
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
                        border: Border::default()
                            .rounded(8)
                            .width(1)
                            .color(palette.background.scale_alpha(0.5)),
                        icon: palette.text,
                        placeholder: palette.text.scale_alpha(0.3),
                        value: palette.text,
                        selection: palette.primary.scale_alpha(0.7),
                    }
                })
        ]
        .spacing(10)
        .align_x(Alignment::Center),
    )
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
        std::fs::File::open("/usr/share/sounds/lofi-alarm-clock.mp3")
            .map_err(NotificationError::FsError)?,
    );
    let sink = rodio::Sink::try_new(&stream_handle).map_err(NotificationError::PlayError)?;
    let source = rodio::Decoder::new_mp3(file).unwrap();
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

fn top_bar<'a>() -> Container<'a, Msg> {
    container(
        custom_button(plus_icon(), CustomButtonType::Primary, None, None).on_press(Msg::AddTimer),
    )
    .padding(10)
    .width(Length::Fill)
    .align_y(Alignment::Start)
    .align_x(Alignment::End)
}

fn scrollable_content<'a>(content: impl Into<Element<'a, Msg>>) -> Scrollable<'a, Msg> {
    scrollable(content)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(10).scroller_width(5),
        ))
        .style(|theme: &Theme, _| {
            let palette = theme.palette();
            let rail_style = scrollable::Rail {
                background: Some(palette.background.into()),
                border: Border::default(),
                scroller: scrollable::Scroller {
                    color: palette.primary,
                    border: Border::default().rounded(8),
                },
            };
            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: rail_style,
                horizontal_rail: rail_style,
                gap: None,
            }
        })
}

fn start_icon<'a>() -> Text<'a> {
    icon('\u{e802}')
}

fn plus_icon<'a>() -> Text<'a> {
    icon('\u{e800}')
}

fn pause_icon<'a>() -> Text<'a> {
    icon('\u{e803}')
}

fn reset_icon<'a>() -> Text<'a> {
    icon('\u{e801}')
}

fn delete_icon<'a>() -> Text<'a> {
    icon('\u{e804}')
}

fn icon<'a>(codepoint: char) -> Text<'a> {
    const ICON_FONT: Font = Font::with_name("icons-font");
    text(codepoint).font(ICON_FONT)
}
