use components::{
    custom_button, delete_icon, pause_icon, reset_icon, save_icon, scrollable_content, start_icon,
    time_container, top_bar, CustomButtonType,
};
use iced::{
    alignment::Horizontal,
    theme,
    widget::{center, column, container, horizontal_space, row},
    Alignment, Border, Element, Length, Shadow, Subscription, Task, Theme,
};
use std::io::{BufReader, BufWriter, Write};
use std::{fs::File, time::Duration};
use uuid::Uuid;

mod components;
mod custom_theme;
mod timer;
mod utils;

fn main() -> iced::Result {
    iced::application("Oxyclock", Oxyclock::update, Oxyclock::view)
        .theme(Oxyclock::theme)
        .subscription(Oxyclock::subscription)
        .font(include_bytes!("../resources/fonts/icons-font.ttf").as_slice())
        .run_with(Oxyclock::load_state)
}

#[derive(Debug, Clone)]
enum Msg {
    AddTimer,
    SaveTimer(Uuid),
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

struct Oxyclock {
    timers: Vec<timer::Timer>,
}

impl Default for Oxyclock {
    fn default() -> Self {
        Oxyclock {
            timers: vec![timer::Timer::default()],
        }
    }
}

impl Oxyclock {
    fn view(&self) -> Element<'_, Msg> {
        let mut timers_container = column![].width(Length::Fill).align_x(Horizontal::Center);
        for timer in self.timers.clone() {
            let started = timer.state == timer::State::Running;

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
                let (hours, minutes, seconds) = timer.time_to_hms_string();
                time_container(timer.id, &timer.name, hours, minutes, seconds, true)
            } else {
                time_container(
                    timer.id,
                    &timer.name,
                    timer.hours,
                    timer.minutes,
                    timer.seconds,
                    false,
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
            .align_left(Length::Fill);

            let save_button = container(
                custom_button(
                    save_icon().size(14f32),
                    CustomButtonType::Success,
                    Some(30f32),
                    Some(30f32),
                )
                .on_press(Msg::SaveTimer(timer.id)),
            )
            .align_right(Length::Fill);

            let timer_container = container(column![
                container(
                    column![
                        if started {
                            row![].height(30)
                        } else {
                            row![delete_button, save_button].width(Length::Fill)
                        },
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

            timers_container = timers_container.push(timer_container);
        }

        container(center(
            column![
                top_bar(),
                scrollable_content(timers_container),
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
                self.timers.push(timer::Timer::new(uuid::Uuid::new_v4()));
                self.save_state(&self.timers);
                Task::none()
            }
            Msg::SaveTimer(id) => {
                let (index, timer) = self
                    .timers
                    .iter()
                    .enumerate()
                    .find(|(_, t)| t.id == id)
                    .unwrap();

                let (mut state, task) = Oxyclock::load_state();
                state.timers[index] = timer.clone();
                self.save_state(&state.timers);

                task
            }
            Msg::DeleteTimer(id) => {
                let index = self.timers.iter().position(|t| t.id == id).unwrap();
                self.timers.remove(index);
                self.save_state(&self.timers);
                Task::none()
            }
            Msg::Start(id) => {
                let timer = self.timers.iter_mut().find(|x| x.id == id).unwrap();
                let duration = timer.get_duration();
                if let Ok(duration) = duration {
                    timer.state = timer::State::Running;
                    timer.time = duration;
                    timer.elapsed = Duration::from_secs(0);
                }
                Task::none()
            }
            Msg::Stop(id) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.state = timer::State::Stopped;
                timer.update_elapsed_hms();
                Task::none()
            }
            Msg::Reset(id) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.state = timer::State::Stopped;
                timer.time = Duration::from_secs(0);
                timer.update_elapsed_hms();
                Task::none()
            }
            Msg::PlayNotification(id) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();
                timer.state = timer::State::NotificationSound;
                Task::done(Msg::Stop(id))
            }
            Msg::Tick(id) => {
                let timer = self.timers.iter_mut().find(|t| t.id == id).unwrap();

                if timer.state != timer::State::Running {
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
                self.save_state(&self.timers);
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
        let timers: Vec<timer::Timer> = serde_json::from_reader(reader).unwrap();
        let state = Oxyclock { timers };
        (state, Task::none())
    }

    fn save_state(&self, timers: &Vec<timer::Timer>) {
        // Since I don't care about Windows
        #[allow(deprecated)]
        let mut path = std::env::home_dir().unwrap();
        path.push(std::path::Path::new(".local/state/oxyclock/state.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, timers).unwrap();
        writer.flush().unwrap();
    }
}
