use iced::widget::{button, container, scrollable};
use iced::{border::Radius, Background, Border, Color, Shadow, Theme, Vector};

/// Styling for components, currently only dark theme is supported
pub fn rounded_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.21))),
            border: Border {
                color: Color::from_rgba(0.35, 0.35, 0.4, 0.4),
                width: 1.5,
                radius: Radius::from(12.0),
            },
            text_color: Color::from_rgb(0.85, 0.85, 0.85),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            },
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.24, 0.24, 0.26))),
            border: Border {
                color: Color::from_rgba(0.45, 0.45, 0.5, 0.6),
                width: 1.5,
                radius: Radius::from(12.0),
            },
            text_color: Color::WHITE,
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.16, 0.16, 0.17))),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.35, 0.5),
                width: 1.5,
                radius: Radius::from(12.0),
            },
            text_color: Color::from_rgb(0.7, 0.7, 0.7),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: Border {
                color: Color::from_rgba(0.2, 0.2, 0.2, 0.3),
                width: 1.5,
                radius: Radius::from(12.0),
            },
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn compact_icon_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.21))),
            border: Border {
                color: Color::from_rgba(0.35, 0.35, 0.4, 0.4),
                width: 1.0,
                radius: Radius::from(10.0),
            },
            text_color: Color::from_rgb(0.85, 0.85, 0.85),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.26, 0.26, 0.28))),
            border: Border {
                color: Color::from_rgba(0.5, 0.5, 0.55, 0.7),
                width: 1.0,
                radius: Radius::from(10.0),
            },
            text_color: Color::WHITE,
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 1.5),
                blur_radius: 4.0,
            },
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.16, 0.16, 0.17))),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.35, 0.5),
                width: 1.0,
                radius: Radius::from(10.0),
            },
            text_color: Color::from_rgb(0.7, 0.7, 0.7),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                offset: Vector::new(0.0, 0.5),
                blur_radius: 1.0,
            },
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: Border {
                color: Color::from_rgba(0.2, 0.2, 0.2, 0.3),
                width: 1.0,
                radius: Radius::from(10.0),
            },
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn selected_gpu_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.28, 0.28, 0.30))),
            border: Border {
                color: Color::from_rgba(0.6, 0.6, 0.65, 0.8),
                width: 1.5,
                radius: Radius::from(10.0),
            },
            text_color: Color::WHITE,
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 5.0,
            },
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.32, 0.32, 0.34))),
            border: Border {
                color: Color::from_rgba(0.7, 0.7, 0.75, 0.9),
                width: 1.5,
                radius: Radius::from(10.0),
            },
            text_color: Color::WHITE,
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: Vector::new(0.0, 2.5),
                blur_radius: 6.0,
            },
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.24, 0.24, 0.26))),
            border: Border {
                color: Color::from_rgba(0.5, 0.5, 0.55, 0.7),
                width: 1.5,
                radius: Radius::from(10.0),
            },
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            },
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: Border {
                color: Color::from_rgba(0.2, 0.2, 0.2, 0.3),
                width: 1.0,
                radius: Radius::from(10.0),
            },
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn card_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.18, 0.18, 0.19))),
        border: Border {
            color: Color::from_rgba(0.4, 0.4, 0.45, 0.5),
            width: 2.0,
            radius: Radius::from(15.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        snap: false,
    }
}

pub fn header_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.18, 0.18, 0.19))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 2.0,
            radius: Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 15.0,
                bottom_right: 15.0,
            },
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        snap: false,
    }
}

pub fn header_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(8.0),
            },
            text_color: Color::from_rgb(0.85, 0.85, 0.85),
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.35, 0.3))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(8.0),
            },
            text_color: Color::WHITE,
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.25, 0.4))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(8.0),
            },
            text_color: Color::from_rgb(0.75, 0.75, 0.75),
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(8.0),
            },
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn modal_generic(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.18, 0.18, 0.19))),
        border: Border {
            color: Color::from_rgba(0.4, 0.4, 0.45, 0.5),
            width: 2.0,
            radius: Radius::from(10.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        snap: false,
    }
}

pub fn thin_scrollbar_style(_theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: Color::from_rgba(0.5, 0.5, 0.5, 0.3),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: Radius::from(2.0),
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: Color::from_rgba(0.5, 0.5, 0.5, 0.3),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: Radius::from(2.0),
                },
            },
        },
        gap: None,
    }
}

pub fn ghost_icon_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(8.0),
            },
            text_color: Color::from_rgb(0.85, 0.85, 0.85),
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                width: 1.0,
                radius: Radius::from(8.0),
            },
            text_color: Color::WHITE,
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(8.0),
            },
            text_color: Color::from_rgb(0.75, 0.75, 0.75),
            shadow: Shadow::default(),
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            text_color: Color::from_rgb(0.4, 0.4, 0.4),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

pub fn stats_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.22, 0.22, 0.24, 0.6))),
        border: Border {
            color: Color::from_rgba(0.3, 0.3, 0.35, 0.3),
            width: 1.0,
            radius: Radius::from(8.0),
        },
        shadow: Shadow::default(),
        text_color: Some(Color::from_rgb(0.7, 0.7, 0.7)),
        snap: false,
    }
}
