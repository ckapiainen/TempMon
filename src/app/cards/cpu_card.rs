use crate::app::settings::Settings;
use crate::app::styles;
use crate::assets;
use crate::collectors::cpu_data::CpuData;
use crate::constants::animation::*;
use crate::types::TempUnits;
use iced::widget::{button, column, container, rich_text, row, rule, span, svg, text};
use iced::{font, never, Center, Color, Element, Fill, Font, Padding};

use crate::app::main_window::MainWindowMessage;

/// # Args
/// * `cpu_data` - CPU statistics and sensor data
/// * `settings` - User settings (temperature units, etc.)
/// * `animation_factor` - Animation progress (0.0 = collapsed, 1.0 = expanded)
/// * `is_expanded` - Whether the card is currently expanded
/// * `on_toggle` - Message to send when the header is clicked
pub fn render_general_cpu_card<'a>(
    cpu_data: &'a CpuData,
    settings: &'a Settings,
    animation_factor: f32,
    is_expanded: bool,
    on_toggle: MainWindowMessage,
) -> Element<'a, MainWindowMessage> {
    // Calculate animated height
    let cpu_card_height = CPU_CARD_COLLAPSED_HEIGHT
        + (animation_factor * (CPU_CARD_EXPANDED_HEIGHT - CPU_CARD_COLLAPSED_HEIGHT));

    // Clickable header
    let cpu_header_button = button(
        row![
            svg(svg::Handle::from_memory(assets::CPU_ICON))
                .width(25)
                .height(25),
            rich_text([span(&cpu_data.name).font(Font {
                weight: font::Weight::Bold,
                ..Font::default()
            }),])
            .on_link_click(never)
            .size(17),
        ]
        .spacing(10)
        .align_y(Center)
        .padding(Padding {
            top: 10.0,
            right: 10.0,
            bottom: 0.0,
            left: 10.0,
        }),
    )
    .on_press(on_toggle)
    .width(Fill)
    .style(styles::header_button_style);

    let cpu_card_content = if is_expanded {
        // Expanded view - show full stats
        let total_load = column![
            text("LOAD").size(20),
            text(format!("{:.2}%", cpu_data.usage)).size(55),
            container(
                column![
                    row![
                        text(format!("L: {:.2}%", cpu_data.usage_min))
                            .size(16)
                            .color(Color::from_rgb(0.7, 0.7, 0.7)),
                        text(" | ").size(16).color(Color::from_rgb(0.7, 0.7, 0.7)),
                        text(format!("H: {:.2}%", cpu_data.usage_max))
                            .size(16)
                            .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    ]
                    .spacing(4),
                    text(format!("Avg: {:.2}%", cpu_data.get_usage_avg()))
                        .size(16)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                ]
                .spacing(3)
                .align_x(Center)
            )
            .padding(8)
            .style(styles::stats_container_style),
        ]
        .align_x(Center)
        .width(195);

        let temp = column![
            text("TEMP").size(20),
            rich_text![
                span(format!(
                    "{:.1}",
                    TempUnits::Celsius.convert(cpu_data.temp, settings.temp_unit())
                ))
                .size(55),
                span(" \u{00B0}").size(38).font(Font {
                    weight: font::Weight::Light,
                    ..Font::default()
                }),
                span(match settings.temp_unit() {
                    TempUnits::Celsius => "C",
                    TempUnits::Fahrenheit => "F",
                })
                .font(Font {
                    weight: font::Weight::Light,
                    ..Font::default()
                })
                .size(35),
            ]
            .on_link_click(never),
            container(
                column![
                    row![
                        text(format!("L: {}", settings.format_temp(cpu_data.temp_min, 1)))
                            .size(16)
                            .color(Color::from_rgb(0.7, 0.7, 0.7)),
                        text(" | ").size(16).color(Color::from_rgb(0.7, 0.7, 0.7)),
                        text(format!("H: {}", settings.format_temp(cpu_data.temp_max, 1)))
                            .size(16)
                            .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    ]
                    .spacing(4),
                    text(format!(
                        "Avg: {}",
                        settings.format_temp(cpu_data.get_temp_avg(), 1)
                    ))
                    .size(16)
                    .color(Color::from_rgb(0.7, 0.7, 0.7)),
                ]
                .spacing(3)
                .align_x(Center)
            )
            .padding(8)
            .style(styles::stats_container_style),
        ]
        .align_x(Center)
        .width(215);

        let clock_speed = column![
            text("CLOCK SPEED").size(18),
            text(format!("{:.0} MHz", cpu_data.current_frequency * 1000.0)).size(38),
            container(rule::horizontal(1)).padding(Padding {
                top: 8.0,
                right: 0.0,
                bottom: 8.0,
                left: 0.0,
            }),
            text("PACKAGE POWER").size(18),
            text(format!("{:.1} W", cpu_data.total_power_draw)).size(38)
        ]
        .align_x(Center)
        .width(190);

        let stats_row = row![
            total_load,
            rule::vertical(1),
            temp,
            rule::vertical(1),
            clock_speed
        ]
        .spacing(25)
        .align_y(Center)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 10.0,
            left: 0.0,
        });

        column![cpu_header_button, rule::horizontal(1), stats_row]
            .align_x(Center)
            .spacing(15)
    } else {
        // Collapsed view - show header with key metrics in one line
        let collapsed_info = row![
            text(settings.format_temp(cpu_data.temp, 0)).size(25),
            text("|").size(25),
            text(format!("{:.1}%", cpu_data.usage)).size(25),
        ]
        .spacing(10)
        .align_y(Center)
        .padding(Padding {
            top: 10.0,
            right: 10.0,
            bottom: 10.0,
            left: 10.0,
        });

        column![row![cpu_header_button, collapsed_info,]
            .width(Fill)
            .align_y(Center)]
    };

    container(cpu_card_content)
        .width(Fill)
        .height(cpu_card_height)
        .align_x(Center)
        .style(styles::card_container_style)
        .clip(true)
        .into()
}
