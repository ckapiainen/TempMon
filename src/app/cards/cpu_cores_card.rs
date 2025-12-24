use crate::app::styles;
use crate::assets;
use crate::constants::animation::*;
use crate::types::{CpuBarChartState, CpuCoreLHMQuery};
use iced::widget::{
    button, column, container, progress_bar, rich_text, row, rule, scrollable, span, svg, text, Row,
};
use iced::{font, never, Center, Element, Fill, Font};

use crate::app::main_window::MainWindowMessage;

/// # Args
/// * `core_usage_vector` - Per-core CPU usage percentages
/// * `core_power_draw_vector` - Per-core power draw in watts
/// * `cpu_bar_chart_state` - Current chart mode (Usage or Power)
/// * `animation_factor` - Animation progress (0.0 = collapsed, 1.0 = expanded)
/// * `is_expanded` - Whether the card is currently expanded
/// * `on_toggle` - Message to send when the header is clicked
pub fn render_cores_card<'a>(
    core_usage_vector: &'a Vec<CpuCoreLHMQuery>,
    core_power_draw_vector: &'a Vec<CpuCoreLHMQuery>,
    cpu_bar_chart_state: CpuBarChartState,
    animation_factor: f32,
    is_expanded: bool,
    on_toggle: MainWindowMessage,
) -> Element<'a, MainWindowMessage> {
    // Calculate animated height
    let cores_card_height = CORES_CARD_COLLAPSED_HEIGHT
        + (animation_factor * (CORES_CARD_EXPANDED_HEIGHT - CORES_CARD_COLLAPSED_HEIGHT));

    // Build usage bar chart
    let usage_bar_chart = build_usage_bar_chart(core_usage_vector);
    let core_usage_row = Row::with_children(usage_bar_chart).spacing(1);

    // Build power bar chart
    let power_bar_chart = build_power_bar_chart(core_power_draw_vector, core_usage_vector.len());
    let core_power_row = Row::with_children(power_bar_chart).spacing(1);

    // Icon buttons for usage and power
    let usage_button = button(
        container(
            svg(svg::Handle::from_memory(assets::MICROCHIP_ICON))
                .width(25)
                .height(25),
        )
        .align_x(Center)
        .align_y(Center)
        .width(25)
        .height(25),
    )
    .on_press(MainWindowMessage::UsageButtonPressed)
    .style(styles::compact_icon_button_style);

    let power_button = button(
        container(
            svg(svg::Handle::from_memory(assets::PLUG_ZAP_ICON))
                .width(25)
                .height(25),
        )
        .align_x(Center)
        .align_y(Center)
        .width(25)
        .height(25),
    )
    .on_press(MainWindowMessage::PowerButtonPressed)
    .style(styles::compact_icon_button_style);

    // Clickable header
    let cores_header_button = button(text("CORES").size(15).font(Font {
        weight: font::Weight::Bold,
        ..Font::default()
    }))
    .on_press(on_toggle)
    .width(Fill)
    .style(styles::header_button_style);

    let cores_card_content: Element<'a, MainWindowMessage> = if is_expanded {
        // Expanded view - show full progress bars with horizontal scrolling
        let header_row = row![cores_header_button, usage_button, power_button,]
            .align_y(Center)
            .spacing(8)
            .width(Fill);

        let scrollable_bars = scrollable(match cpu_bar_chart_state {
            CpuBarChartState::Usage => core_usage_row,
            CpuBarChartState::Power => core_power_row,
        })
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().scroller_width(4),
        ));

        column![header_row, rule::horizontal(1), scrollable_bars]
            .align_x(Center)
            .spacing(10)
            .padding(10)
            .into()
    } else {
        // Collapsed view - show summary with buttons
        let mode_text = match cpu_bar_chart_state {
            CpuBarChartState::Usage => "Usage",
            CpuBarChartState::Power => "Power",
        };

        let collapsed_info = row![
            text(format!("{} cores", core_usage_vector.len())).size(14),
            text("|").size(14),
            text(mode_text).size(14),
        ]
        .spacing(10);

        column![row![
            cores_header_button,
            collapsed_info,
            usage_button,
            power_button,
        ]
        .align_y(Center)
        .spacing(8)
        .width(Fill)]
        .padding(10)
        .into()
    };

    container(cores_card_content)
        .width(Fill)
        .height(cores_card_height)
        .align_x(Center)
        .style(styles::card_container_style)
        .clip(true)
        .into()
}

/// Builds the usage bar chart with vertical progress bars for each core.
fn build_usage_bar_chart(
    core_usage_vector: &Vec<CpuCoreLHMQuery>,
) -> Vec<Element<MainWindowMessage>> {
    let mut usage_bar_chart: Vec<Element<MainWindowMessage>> = Vec::new();

    for (i, core) in core_usage_vector.iter().enumerate() {
        let utilization = progress_bar(0.0..=100.0, core.value)
            .vertical()
            .length(150)
            .girth(28);

        let name_util_val = rich_text![
            span(format!("{:.2}%\n", core.value))
                .font(Font {
                    weight: font::Weight::Thin,
                    ..Font::default()
                })
                .size(15),
            span(core.name.to_string())
                .font(Font {
                    weight: font::Weight::Thin,
                    ..Font::default()
                })
                .size(15),
        ]
        .on_link_click(never)
        .align_x(Center)
        .width(55);

        let core_col = column![utilization, name_util_val].align_x(Center);
        usage_bar_chart.push(core_col.into());

        // Add vertical rule between cores but not after the last one
        if i < core_usage_vector.len() - 1 {
            usage_bar_chart.push(rule::vertical(1).into());
        }
    }

    usage_bar_chart
}

/// Builds the power bar chart with vertical progress bars for each core.
fn build_power_bar_chart(
    core_power_draw_vector: &Vec<CpuCoreLHMQuery>,
    core_count: usize,
) -> Vec<Element<MainWindowMessage>> {
    let mut power_bar_chart: Vec<Element<MainWindowMessage>> = Vec::new();

    for (i, core) in core_power_draw_vector.iter().enumerate() {
        let wattage_bar = progress_bar(0.0..=20.0, core.value)
            .vertical()
            .length(150)
            .girth(28);

        let name_util_val = rich_text![
            span(format!("{:.2}W\n", core.value))
                .font(Font {
                    weight: font::Weight::Thin,
                    ..Font::default()
                })
                .size(15),
            span(core.name.replace("#", "").to_string())
                .font(Font {
                    weight: font::Weight::Thin,
                    ..Font::default()
                })
                .size(15),
        ]
        .on_link_click(never)
        .align_x(Center)
        .width(55);

        let core_col = column![wattage_bar, name_util_val].align_x(Center);
        power_bar_chart.push(core_col.into());

        // Add vertical rule between cores but not after the last one
        if i < core_count - 1 {
            power_bar_chart.push(rule::vertical(1).into());
        }
    }

    power_bar_chart
}
