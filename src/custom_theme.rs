use iced::{
    theme::{self, Palette},
    Color,
};

pub fn arc_dark() -> theme::Theme {
    theme::Theme::custom(
        "Arc-Dark".to_string(),
        Palette {
            background: Color::from_rgb(47.0 / 255.0, 52.0 / 255.0, 63.0 / 255.0),
            text: Color::from_rgb(211.0 / 255.0, 218.0 / 255.0, 227.0 / 255.0),
            primary: Color::from_rgb(82.0 / 255.0, 148.0 / 255.0, 226.0 / 255.0),
            success: Color::from_rgb(155.0 / 255.0, 89.0 / 255.0, 182.0 / 255.0),
            danger: Color::from_rgb(220.0 / 255.0, 50.0 / 255.0, 47.0 / 255.0),
        },
    )
}
