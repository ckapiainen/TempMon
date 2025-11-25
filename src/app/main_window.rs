use crate::app::styles;
use crate::collectors::cpu_collector::CpuData;
use iced::widget::{
    button, column, container, progress_bar, rich_text, row, rule, span, svg, text, Row,
};
use iced::{font, never, window, Center, Element, Fill, Font, Padding, Subscription};
use lilt::{Animated, Easing};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarChartState {
    Usage,
    Power,
}

#[derive(Debug, Clone)]
pub enum MainWindowMessage {
    UsageButtonPressed,
    PowerButtonPressed,
    // Animation triggers
    ToggleGeneralInfo,
    ToggleCoresCard,
    Tick, // Frame update (REQUIRED for animations)
}

pub struct MainWindow {
    bar_chart_state: BarChartState,
    general_info_expanded: Animated<f32, Instant>,
    cores_card_expanded: Animated<f32, Instant>,
    now: Instant,
}

//TODO: Responsive layout: max size for cards and move them according to screen size (switch between column/row or some better way with iced api)
// TODO: 5 sec timeout before setting min/max values. 100% max value clips with box next to it
impl MainWindow {
    pub fn new() -> Self {
        Self {
            bar_chart_state: BarChartState::Usage,
            general_info_expanded: Animated::new(1.0).duration(400.0).easing(Easing::EaseInOut),
            cores_card_expanded: Animated::new(1.0).duration(400.0).easing(Easing::EaseInOut),
            now: Instant::now(),
        }
    }

    pub fn update(&mut self, message: MainWindowMessage) {
        match message {
            MainWindowMessage::UsageButtonPressed => {
                self.bar_chart_state = BarChartState::Usage;
            }
            MainWindowMessage::PowerButtonPressed => {
                self.bar_chart_state = BarChartState::Power;
            }
            MainWindowMessage::ToggleGeneralInfo => {
                // 0.0 Collapsed, 1.0 Expanded
                let new_value = if self.general_info_expanded.value > 0.5 {
                    0.0
                } else {
                    1.0
                };
                // Start the transition
                self.general_info_expanded
                    .transition(new_value, Instant::now());
            }
            MainWindowMessage::ToggleCoresCard => {
                let new_value = if self.cores_card_expanded.value > 0.5 {
                    0.0
                } else {
                    1.0
                };
                self.cores_card_expanded
                    .transition(new_value, Instant::now());
            }
            MainWindowMessage::Tick => {
                // Update current time on each frame
                self.now = Instant::now();
            }
        }
    }

    pub fn subscription(&self) -> Subscription<MainWindowMessage> {
        // Only subscribe to frames when animations are active
        if self.general_info_expanded.in_progress(self.now)
            || self.cores_card_expanded.in_progress(self.now)
        {
            window::frames().map(|_| MainWindowMessage::Tick)
        } else {
            Subscription::none()
        }
    }

    pub fn view<'a>(&self, cpu_data: &'a CpuData) -> Element<'a, MainWindowMessage> {
        let core_usage_vector = &cpu_data.core_utilization;
        let core_power_draw_vector = &cpu_data.core_power_draw;

        /*
        General CPU info card
        */

        // Animate height between collapsed (50px) and expanded (260px)
        // 1.0 = expanded, 0.0 = collapsed
        let animation_factor = self
            .general_info_expanded
            .animate(std::convert::identity, self.now);
        let general_card_height = 50.0 + (animation_factor * (260.0 - 50.0));
        let is_general_expanded = self.general_info_expanded.value > 0.5;

        // Clickable header
        let general_header_button = button(
            row![
                svg(svg::Handle::from_path("assets/icons/cpu.svg"))
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
        .on_press(MainWindowMessage::ToggleGeneralInfo)
        .width(Fill)
        .style(styles::header_button_style);

        let general_content = if is_general_expanded {
            // Expanded view - show full stats
            let total_load = column![
                text("LOAD").size(20),
                text(format!("{:.2}%", cpu_data.usage)).size(55),
                row![
                    text(format!("L: {:.2}%", cpu_data.usage_low)).size(20),
                    text(" | ").size(20),
                    text(format!("H: {:.2}%", cpu_data.usage_high)).size(20),
                ]
                .spacing(5)
            ]
            .align_x(Center)
            .width(195);

            let temp = column![
                text("TEMP").size(20),
                rich_text![
                    span(format!("{:.1}", cpu_data.temp)).size(55),
                    span(" \u{00B0}").size(38).font(Font {
                        weight: font::Weight::Light,
                        ..Font::default()
                    }),
                    span("C")
                        .font(Font {
                            weight: font::Weight::Light,
                            ..Font::default()
                        })
                        .size(35),
                ]
                .on_link_click(never),
                row![
                    text(format!("L: {:.2}°C", cpu_data.temp_low)).size(20),
                    text(" | ").size(20),
                    text(format!("H: {:.2}°C", cpu_data.temp_high)).size(20),
                ]
                .spacing(5)
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

            column![general_header_button, rule::horizontal(1), stats_row]
                .align_x(Center)
                .spacing(15)
        } else {
            // Collapsed view - show header with key metrics in one line
            let collapsed_info = row![
                text(format!("{}°C", cpu_data.temp as i32)).size(25),
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

            column![row![general_header_button, collapsed_info,]
                .width(Fill)
                .align_y(Center)]
        };

        let general_cpu_info_card = container(general_content)
            .width(Fill)
            .height(general_card_height)
            .align_x(Center)
            .style(styles::card_container_style)
            .clip(true);

        /*
          CORE USAGE COLUMNS
        */

        // Build core row with vertical rules between cores
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

        /*
          CORE POWER DRAW COLUMNS
        */
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
            if i < core_usage_vector.len() - 1 {
                power_bar_chart.push(rule::vertical(1).into());
            }
        }
        let core_usage_row = Row::with_children(usage_bar_chart).spacing(1);
        let core_power_row = Row::with_children(power_bar_chart).spacing(1);

        /*
          Cores card with collapse functionality
        */

        // Animate height between collapsed (50px) and expanded (280px)
        // 1.0 = expanded, 0.0 = collapsed
        let cores_animation_factor = self
            .cores_card_expanded
            .animate(std::convert::identity, self.now);
        let cores_card_height = 50.0 + (cores_animation_factor * (280.0 - 50.0));
        let is_cores_expanded = self.cores_card_expanded.value > 0.5;

        // Icon buttons for usage and power
        let usage_button = button(
            container(
                svg(svg::Handle::from_path("assets/icons/microchip.svg"))
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
                svg(svg::Handle::from_path("assets/icons/plug-zap.svg"))
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
        .on_press(MainWindowMessage::ToggleCoresCard)
        .width(Fill)
        .style(styles::header_button_style);

        let cores_card_content = if is_cores_expanded {
            // Expanded view - show full progress bars
            let header_row = row![cores_header_button, usage_button, power_button,]
                .align_y(Center)
                .spacing(8)
                .width(Fill);

            column![
                header_row,
                rule::horizontal(1),
                match self.bar_chart_state {
                    BarChartState::Usage => core_usage_row,
                    BarChartState::Power => core_power_row,
                }
            ]
            .align_x(Center)
            .spacing(10)
            .padding(10)
        } else {
            // Collapsed view - show summary with buttons
            let mode_text = match self.bar_chart_state {
                BarChartState::Usage => "Usage",
                BarChartState::Power => "Power",
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
        };

        let cores_card = container(cores_card_content)
            .width(Fill)
            .height(cores_card_height)
            .align_x(Center)
            .style(styles::card_container_style)
            .clip(true);

        let all_cards = column![general_cpu_info_card, cores_card].spacing(20);
        container(all_cards).padding(20).width(Fill).into()
    }
}
