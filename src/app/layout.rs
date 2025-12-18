use crate::app::styles;
use crate::app::tempmon::{Screen, TempMonMessage};
use crate::assets;
use iced::widget::{button, column, container, row, svg};
use iced::{Center, Element, Fill};

/// Render the app with header
pub fn with_header<'a>(
    content: Element<'a, TempMonMessage>,
    current_screen: &Screen,
) -> Element<'a, TempMonMessage> {
    let main_page_button = button(
        container(
            svg(svg::Handle::from_memory(assets::MENU_ICON))
                .width(30)
                .height(30),
        )
        .align_x(Center)
        .align_y(Center)
        .width(35)
        .height(35),
    )
    .on_press(TempMonMessage::MainButtonPressed)
    .style(if matches!(current_screen, Screen::Main) {
        styles::active_header_button_style
    } else {
        styles::rounded_button_style
    });

    let plotter_page = button(
        container(
            svg(svg::Handle::from_memory(assets::CHART_SPLINE_ICON))
                .width(30)
                .height(30),
        )
        .align_x(Center)
        .align_y(Center)
        .width(35)
        .height(35),
    )
    .on_press(TempMonMessage::PlotterButtonPressed)
    .style(if matches!(current_screen, Screen::Plotter) {
        styles::active_header_button_style
    } else {
        styles::rounded_button_style
    });

    let settings_page = button(
        container(
            svg(svg::Handle::from_memory(assets::SETTINGS_ICON))
                .width(30)
                .height(30),
        )
        .align_x(Center)
        .align_y(Center)
        .width(35)
        .height(35),
    )
    .on_press(TempMonMessage::ShowSettingsModal)
    .style(styles::rounded_button_style);

    let header = container(
        row![main_page_button, plotter_page, settings_page]
            .align_y(Center)
            .spacing(8),
    )
    .padding(10)
    .align_x(Center)
    .align_y(Center)
    .style(styles::header_container_style)
    .width(250);

    // center the header horizontally at top
    let header_wrapper = container(header).width(Fill).center_x(Fill);

    container(column![header_wrapper, content].spacing(20))
        .width(Fill)
        .height(Fill)
        .into()
}
