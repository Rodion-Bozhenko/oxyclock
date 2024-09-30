use iced::{
    alignment::{Horizontal, Vertical},
    border,
    widget::{
        button, column, container, row, scrollable, text, text_input, Button, Container,
        Scrollable, Text, TextInput,
    },
    Alignment, Border, Element, Font, Length, Shadow, Theme,
};
use uuid::Uuid;

use crate::{Msg, Time};

const TEXT_SIZE: u16 = 50;

#[derive(PartialEq)]
pub enum CustomButtonType {
    Primary,
    Secondary,
    Success,
}

pub fn custom_button<'a>(
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
                        CustomButtonType::Success => ext_palette.success.strong.color.into(),
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

pub fn time_container<'a>(
    timer_id: Uuid,
    name: &str,
    hours: String,
    minutes: String,
    seconds: String,
    running: bool,
) -> Container<'a, Msg> {
    let time_row = row![
        if running {
            time_text(hours)
        } else {
            time_input(timer_id, &hours, Msg::Hours)
        },
        text(":").size(TEXT_SIZE).align_x(Horizontal::Center),
        if running {
            time_text(minutes)
        } else {
            time_input(timer_id, &minutes, Msg::Minutes)
        },
        text(":").size(TEXT_SIZE).align_x(Horizontal::Center),
        if running {
            time_text(seconds)
        } else {
            time_input(timer_id, &seconds, Msg::Seconds)
        },
    ]
    .height(70)
    .align_y(Vertical::Center);

    container(
        column![time_row, name_input(timer_id, name, true)]
            .spacing(10)
            .align_x(Alignment::Center),
    )
}

fn name_input<'a>(timer_id: Uuid, name: &str, disabled: bool) -> TextInput<'a, Msg> {
    let input = text_input("Name", name)
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
        });
    if disabled {
        input
    } else {
        input.on_input(move |name| Msg::Name((timer_id, name)))
    }
}

fn time_input<'a, F>(timer_id: Uuid, value: &str, msg: F) -> Container<'a, Msg>
where
    F: 'static + Fn(Time) -> Msg,
{
    container(
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
            }),
    )
}

fn time_text<'a>(t: String) -> Container<'a, Msg> {
    container(
        text(t)
            .width(70)
            .height(70f32 + text_input::DEFAULT_PADDING.top)
            .size(TEXT_SIZE)
            .align_y(Alignment::Center)
            .align_x(Alignment::Center),
    )
    .style(|theme: &Theme| container::Style {
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
        border: Border::default()
            .rounded(8)
            .width(1)
            .color(theme.palette().background.scale_alpha(0.5)),
        shadow: Shadow::default(),
    })
}

pub fn top_bar<'a>() -> Container<'a, Msg> {
    container(
        custom_button(plus_icon(), CustomButtonType::Primary, None, None).on_press(Msg::AddTimer),
    )
    .padding(10)
    .width(Length::Fill)
    .align_y(Alignment::Start)
    .align_x(Alignment::End)
}

pub fn scrollable_content<'a>(content: impl Into<Element<'a, Msg>>) -> Scrollable<'a, Msg> {
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

pub fn start_icon<'a>() -> Text<'a> {
    icon('\u{e802}')
}

pub fn plus_icon<'a>() -> Text<'a> {
    icon('\u{e800}')
}

pub fn pause_icon<'a>() -> Text<'a> {
    icon('\u{e803}')
}

pub fn reset_icon<'a>() -> Text<'a> {
    icon('\u{e801}')
}

pub fn delete_icon<'a>() -> Text<'a> {
    icon('\u{e804}')
}

pub fn save_icon<'a>() -> Text<'a> {
    icon('\u{e805}')
}
fn icon<'a>(codepoint: char) -> Text<'a> {
    const ICON_FONT: Font = Font::with_name("icons-font");
    text(codepoint).font(ICON_FONT)
}
