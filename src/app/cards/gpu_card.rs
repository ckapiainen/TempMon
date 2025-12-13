use crate::app::settings::Settings;
use crate::app::styles;
use crate::assets;
use crate::collectors::GpuData;
use crate::constants::animation::*;
use crate::types::TempUnits;
use iced::widget::{button, column, container, rich_text, row, rule, span, svg, text, Row};
use iced::{font, never, Center, Color, Element, Fill, Font, Padding, Theme};

use crate::app::main_window::MainWindowMessage;

/// Returns `None` if no GPU data is available.
/// # Args
/// * `gpu_data` - Vector of GPU statistics (supports multiple GPUs)
/// * `settings` - User settings (temperature units, etc.)
/// * `selected_gpu_index` - Index of the currently selected GPU
/// * `animation_factor` - Animation progress (0.0 = collapsed, 1.0 = expanded)
/// * `is_expanded` - Whether the card is currently expanded
/// * `on_toggle` - Message to send when the header is clicked
pub fn render_gpu_card<'a>(
    gpu_data: &'a Vec<GpuData>,
    settings: &'a Settings,
    selected_gpu_index: usize,
    animation_factor: f32,
    is_expanded: bool,
    on_toggle: MainWindowMessage,
) -> Option<Element<'a, MainWindowMessage>> {
    // Return None if no GPU data available
    if gpu_data.is_empty() {
        return None;
    }

    // Calculate animated height
    let gpu_card_height = GPU_CARD_COLLAPSED_HEIGHT
        + (animation_factor * (GPU_CARD_EXPANDED_HEIGHT - GPU_CARD_COLLAPSED_HEIGHT));

    // Build GPU switch buttons
    let gpu_switch_button_row =
        render_gpu_switch_buttons(gpu_data, selected_gpu_index, is_expanded);

    // Clickable header with GPU selector buttons
    let gpu_header_button = button(
        row![
            svg(svg::Handle::from_memory(assets::GPU_ICON))
                .width(25)
                .height(25),
            gpu_switch_button_row
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

    let gpu_card_content = if is_expanded {
        // Expanded view - show full stats
        let gpu = get_gpu_safe(gpu_data, selected_gpu_index);

        // Left column: Core Load + Memory Usage
        let memory_used_gb = gpu.memory_used / 1024.0;
        let memory_total_gb = gpu.memory_total / 1024.0;
        let memory_percentage = if gpu.memory_total > 0.0 {
            (gpu.memory_used / gpu.memory_total) * 100.0
        } else {
            0.0
        };

        let left_column = column![
            text("CORE LOAD").size(18),
            text(format!("{:.1}%", gpu.core_load)).size(48),
            container(rule::horizontal(1)).padding(Padding {
                top: 8.0,
                right: 0.0,
                bottom: 8.0,
                left: 0.0,
            }),
            text("MEMORY USAGE").size(16),
            text(format!("{:.1} / {:.1} GB", memory_used_gb, memory_total_gb)).size(24),
            text(format!("({:.1}%)", memory_percentage)).size(18),
        ]
        .align_x(Center)
        .width(160);

        // Middle column: Core Temp + Memory Junction Temp (both with L/A/H)
        let middle_column = column![
            text("CORE TEMP").size(18),
            rich_text![
                span(format!(
                    "{:.1}",
                    TempUnits::Celsius.convert(gpu.core_temp, settings.temp_unit())
                ))
                .size(48),
                span(" \u{00B0}").size(32).font(Font {
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
                .size(30),
            ]
            .on_link_click(never),
            container(
                row![
                    text(format!("L: {}", settings.format_temp(gpu.core_temp_min, 1)))
                        .size(16)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text(" | ").size(16).color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text(format!(
                        "Avg: {}",
                        settings.format_temp(gpu.get_core_temp_avg(), 1)
                    ))
                    .size(16)
                    .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text(" | ").size(16).color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text(format!("H: {}", settings.format_temp(gpu.core_temp_max, 1)))
                        .size(16)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                ]
                .spacing(4)
            )
            .padding(8)
            .style(styles::stats_container_style),
            container(rule::horizontal(1)).padding(Padding {
                top: 8.0,
                right: 0.0,
                bottom: 8.0,
                left: 0.0,
            }),
            text("MEMORY JUNCTION").size(16),
            rich_text![
                span(format!(
                    "{:.1}",
                    TempUnits::Celsius.convert(gpu.memory_junction_temp, settings.temp_unit())
                ))
                .size(48),
                span(" \u{00B0}").size(32).font(Font {
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
                .size(30),
            ]
            .on_link_click(never),
            container(
                row![
                    text(format!(
                        "L: {}",
                        settings.format_temp(gpu.memory_junction_temp_min, 1)
                    ))
                    .size(16)
                    .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text(" | ").size(16).color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text(format!(
                        "Avg: {}",
                        settings.format_temp(gpu.get_memory_junction_temp_avg(), 1)
                    ))
                    .size(16)
                    .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text(" | ").size(16).color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text(format!(
                        "H: {}",
                        settings.format_temp(gpu.memory_junction_temp_max, 1)
                    ))
                    .size(16)
                    .color(Color::from_rgb(0.7, 0.7, 0.7)),
                ]
                .spacing(4)
            )
            .padding(8)
            .style(styles::stats_container_style),
        ]
        .align_x(Center)
        .width(284);

        // Right column: Core Clock + Memory Clock + Package Power
        let right_column = column![
            text("CORE CLOCK").size(16),
            text(format!("{:.0} MHz", gpu.core_clock)).size(32),
            container(rule::horizontal(1)).padding(Padding {
                top: 8.0,
                right: 0.0,
                bottom: 8.0,
                left: 0.0,
            }),
            text("MEMORY CLOCK").size(16),
            text(format!("{:.0} MHz", gpu.memory_clock)).size(32),
            container(rule::horizontal(1)).padding(Padding {
                top: 8.0,
                right: 0.0,
                bottom: 8.0,
                left: 0.0,
            }),
            text("PACKAGE POWER").size(16),
            text(format!("{:.1} W", gpu.power)).size(32)
        ]
        .align_x(Center)
        .width(160);

        let stats_row = row![
            left_column,
            rule::vertical(1),
            middle_column,
            rule::vertical(1),
            right_column
        ]
        .spacing(25)
        .align_y(Center)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 10.0,
            left: 0.0,
        });

        column![gpu_header_button, rule::horizontal(1), stats_row]
            .align_x(Center)
            .spacing(15)
    } else {
        // Collapsed view - show header with key metrics in one line
        let gpu = get_gpu_safe(gpu_data, selected_gpu_index);

        let collapsed_info = row![
            text(settings.format_temp(gpu.core_temp, 1)).size(25),
            text("|").size(25),
            text(settings.format_temp(gpu.memory_junction_temp, 1)).size(25),
            text("|").size(25),
            text(format!("{:.1}%", gpu.core_load)).size(25),
        ]
        .spacing(10)
        .align_y(Center)
        .padding(Padding {
            top: 5.0,
            right: 5.0,
            bottom: 5.0,
            left: 5.0,
        });

        column![row![gpu_header_button, collapsed_info,]
            .width(Fill)
            .align_y(Center)]
    };

    let gpu_card = container(gpu_card_content)
        .width(Fill)
        .height(gpu_card_height)
        .align_x(Center)
        .style(styles::card_container_style)
        .clip(true);

    Some(gpu_card.into())
}

/// Renders the GPU switch buttons for multi-GPU setups.
fn render_gpu_switch_buttons<'a>(
    gpu_data: &'a Vec<GpuData>,
    selected_gpu_index: usize,
    is_expanded: bool,
) -> Row<'a, MainWindowMessage, Theme, iced::Renderer> {
    Row::with_children(
        gpu_data
            .iter()
            .enumerate()
            .map(|(index, gpu)| {
                // Determine button style based on selection
                let button_style = if index == selected_gpu_index {
                    styles::selected_gpu_button_style
                } else {
                    styles::compact_icon_button_style
                };

                // Determine button text based on card state
                let button_text = if is_expanded {
                    format!("{}", gpu.name)
                } else {
                    format!("{}", index)
                };

                button(text(button_text))
                    .on_press(MainWindowMessage::GpuButtonPressed(index))
                    .style(button_style)
                    .into()
            })
            .collect::<Vec<Element<'a, MainWindowMessage, Theme, iced::Renderer>>>(),
    )
    .spacing(8)
    .align_y(Center)
}

/// If the index is out of bounds, logs an error and returns the first GPU,
/// or a default empty GPU if no GPUs are available.
fn get_gpu_safe<'a>(gpu_data: &'a Vec<GpuData>, selected_gpu_index: usize) -> &'a GpuData {
    // Create a default GPU once for fallback
    static DEFAULT_GPU: std::sync::OnceLock<GpuData> = std::sync::OnceLock::new();
    let default_gpu = DEFAULT_GPU
        .get_or_init(|| GpuData::new(lhm_client::HardwareType::GpuNvidia, "No GPU".to_string()));

    match gpu_data.get(selected_gpu_index) {
        Some(gpu) => gpu,
        None => {
            // Index out of bounds - reset to 0 or show error
            eprintln!(
                "GPU index {} out of bounds (len: {})",
                selected_gpu_index,
                gpu_data.len()
            );
            gpu_data.first().unwrap_or(default_gpu)
        }
    }
}
