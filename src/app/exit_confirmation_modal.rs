use crate::app::styles;
use crate::app::tempmon::TempMonMessage;
use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, text, toggler};
use iced::{Alignment, Color, Element, Length};

pub fn exit_confirmation_modal<'a>(
    base: impl Into<Element<'a, TempMonMessage>>,
) -> Element<'a, TempMonMessage> {
    // Header with title and close button
    let header = row![
        text("Close Application")
            .size(24)
            .width(Length::Fill)
            .style(|_theme| text::Style {
                color: Some(Color::from_rgb(0.9, 0.9, 0.9))
            }),
        button(text("✕").size(20))
            .on_press(TempMonMessage::CancelExit)
            .padding(5)
            .style(styles::header_button_style),
    ]
    .align_y(Alignment::Center)
    .spacing(10);

    // Main content
    let content = column![
        header,
        text("Do you want to minimize to the system tray or exit the application?")
            .size(14)
            .style(|_theme| text::Style {
                color: Some(Color::from_rgb(0.75, 0.75, 0.75))
            }),
        // Action buttons
        container(
            row![
                button(
                    text("Minimize")
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                )
                .on_press(TempMonMessage::ConfirmMinimize)
                .padding(12)
                .width(100)
                .style(styles::minimize_button_style),
                button(text("Exit").width(Length::Fill).align_x(Horizontal::Center))
                    .on_press(TempMonMessage::ConfirmExit)
                    .padding(12)
                    .width(100)
                    .style(styles::exit_button_style),
            ]
            .spacing(10),
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center),
        row![
            toggler(false),
            text("Remember my choice")
                .size(12)
                .style(|_theme| text::Style {
                    color: Some(Color::from_rgb(0.6, 0.6, 0.6))
                })
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(15)
    .padding(20);

    let modal_content = container(content).width(400).style(styles::modal_generic);

    crate::app::modal::modal(base, modal_content, TempMonMessage::CancelExit, true)
}
