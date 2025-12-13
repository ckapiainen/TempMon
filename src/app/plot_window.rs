use crate::app::graphs::cpu_power_usage::CPUPowerAndUsageGraph;
use crate::app::graphs::gpu_power_usage::GPUPowerAndUsageGraph;
use crate::app::graphs::temp_graph::TemperatureGraph;
use crate::app::styles;
use crate::app::styles::compact_icon_button_style;
use crate::constants::sidebar::*;
use crate::types::TempUnits;
use crate::utils::csv_logger::CsvLogger;
use iced::widget::{
    button, column, combo_box, container, row, rule, scrollable, svg, text, Column,
};
use iced::{window, Alignment, Center, Color, Element, Length, Subscription, Theme};
use lilt::{Animated, Easing};
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::{Pid, Process};

pub struct PlotWindow {
    temp_graph: TemperatureGraph,
    cpu_power_usage_graph: CPUPowerAndUsageGraph,
    gpu_power_usage_graph: GPUPowerAndUsageGraph,
    // Process monitoring
    selected_processes: Vec<String>,
    // Process sidebar state
    sidebar_expanded: Animated<f32, Instant>,
    now: Instant,
}

#[derive(Debug, Clone)]
pub enum PlotWindowMessage {
    TempPlotMessage(iced_plot::PlotUiMessage),
    CPUPowerUsagePlotMessage(iced_plot::PlotUiMessage),
    GPUPowerUsagePlotMessage(iced_plot::PlotUiMessage),
    Tick,
    ToggleSidebar,
    ProcessSelected(String),
    RemoveProcess(String),
}

// TODO: Process monitor
//TODO: toggle show/hide for gpu

impl PlotWindow {
    pub fn new(temp_units_from_settings: String) -> Self {
        let units = if temp_units_from_settings == "Celsius" {
            TempUnits::Celsius
        } else {
            TempUnits::Fahrenheit
        };

        // Mock data
        let available_processes = vec![
            "chrome.exe".to_string(),
            "rustc.exe".to_string(),
            "code.exe".to_string(),
            "spotify.exe".to_string(),
            "system".to_string(),
            "discord.exe".to_string(),
            "notepad.exe".to_string(),
            "explorer.exe".to_string(),
        ];

        Self {
            temp_graph: TemperatureGraph::new(units),
            cpu_power_usage_graph: CPUPowerAndUsageGraph::new(),
            gpu_power_usage_graph: GPUPowerAndUsageGraph::new(),
            selected_processes: vec![],
            sidebar_expanded: Animated::new(0.0).duration(300.0).easing(Easing::EaseInOut),
            now: Instant::now(),
        }
    }

    pub fn update(
        &mut self,
        csv_logger: &CsvLogger,
        message: PlotWindowMessage,
        units: TempUnits,
        gpu_data: &[crate::collectors::GpuData],
    ) {
        match message {
            PlotWindowMessage::TempPlotMessage(msg) => self.temp_graph.update_ui(msg),
            PlotWindowMessage::CPUPowerUsagePlotMessage(msg) => {
                self.cpu_power_usage_graph.update_ui(msg)
            }
            PlotWindowMessage::GPUPowerUsagePlotMessage(msg) => {
                self.gpu_power_usage_graph.update_ui(msg)
            }
            PlotWindowMessage::Tick => {
                self.now = Instant::now();
                self.temp_graph.update_data(csv_logger, units, gpu_data);
                self.cpu_power_usage_graph.update_data(csv_logger);
                self.gpu_power_usage_graph.update_data(csv_logger, gpu_data);
            }

            // Sidebar controls
            PlotWindowMessage::ToggleSidebar => {
                // 0.0 = collapsed, 1.0 = expanded
                let new_value = if self.sidebar_expanded.value > 0.5 {
                    0.0
                } else {
                    1.0
                };
                self.sidebar_expanded.transition(new_value, Instant::now());
            }
            PlotWindowMessage::ProcessSelected(proc) => {
                if !self.selected_processes.contains(&proc) {
                    self.selected_processes.push(proc.clone());
                }
            }
            PlotWindowMessage::RemoveProcess(proc) => {
                self.selected_processes.retain(|p| p != &proc);
            }
        }
    }

    pub fn subscription(&self) -> Subscription<PlotWindowMessage> {
        // Only sub to frames when animation are active
        if self.sidebar_expanded.in_progress(self.now) {
            window::frames().map(|_| PlotWindowMessage::Tick)
        } else {
            Subscription::none()
        }
    }

    pub fn view<'a>(&'a self, sys: &'a HashMap<Pid, Process>) -> Element<'a, PlotWindowMessage> {
        let sidebar_animation_factor = self
            .sidebar_expanded
            .animate(std::convert::identity, self.now);

        let current_sidebar_width = SIDEBAR_COLLAPSED_WIDTH
            + (sidebar_animation_factor * (SIDEBAR_EXPANDED_WIDTH - SIDEBAR_COLLAPSED_WIDTH));
        let is_collapsed = self.sidebar_expanded.value < 0.5;

        /*
         ========== SIDEBAR CONTENT ==========
        */
        let toggle_icon = if is_collapsed {
            crate::assets::ARROW_RIGHT_ICON // Expand
        } else {
            crate::assets::ARROW_LEFT_ICON // Collapse
        };

        let toggle_btn = button(
            container(
                svg(svg::Handle::from_memory(toggle_icon))
                    .width(30)
                    .height(30),
            )
            .align_x(Center)
            .align_y(Center)
            .width(30)
            .height(30),
        )
        .on_press(PlotWindowMessage::ToggleSidebar)
        .style(styles::ghost_icon_button_style)
        .padding(4);
        let process_column = PlotWindow::process_column(sys);

        // Left column: Selected processes
        let selected_column = scrollable(
            column![
                text("Selected").size(12).style(|_| text::Style {
                    color: Some(Color::from_rgb(0.8, 0.8, 0.8))
                }),
                rule::horizontal(1).style(|_| rule::Style {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    radius: 1.0.into(),
                    fill_mode: rule::FillMode::Percent(100.0),
                    snap: false,
                }),
                // Selected Pills
                if self.selected_processes.is_empty() {
                    column![text("None").size(11).style(|_| text::Style {
                        color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                    })]
                } else {
                    column(
                        self.selected_processes
                            .iter()
                            .map(|proc| {
                                button(
                                    row![text(proc).size(11), text("×").size(12)]
                                        .spacing(4)
                                        .align_y(Center),
                                )
                                .on_press(PlotWindowMessage::RemoveProcess(proc.clone()))
                                .style(compact_icon_button_style)
                                .padding([3, 8])
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .spacing(4)
                }
            ]
            .spacing(8),
        )
        .width(Length::FillPortion(1));

        // Right column: Process list with header
        let process_header = row![
            text("Name")
                .size(10)
                .width(Length::FillPortion(3))
                .style(|_| text::Style {
                    color: Some(Color::from_rgb(0.7, 0.7, 0.7))
                }),
            text("CPU")
                .size(10)
                .width(Length::Fixed(50.0))
                .style(|_| text::Style {
                    color: Some(Color::from_rgb(0.7, 0.7, 0.7))
                }),
            text("Mem")
                .size(10)
                .width(Length::Fixed(55.0))
                .style(|_| text::Style {
                    color: Some(Color::from_rgb(0.7, 0.7, 0.7))
                }),
            text("").size(10).width(Length::Fixed(30.0)), // Space for button column
        ]
        .spacing(5);

        let process_list_column = scrollable(
            column![
                text("Processes").size(12).style(|_| text::Style {
                    color: Some(Color::from_rgb(0.8, 0.8, 0.8))
                }),
                rule::horizontal(1).style(|_| rule::Style {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    radius: 1.0.into(),
                    fill_mode: rule::FillMode::Percent(100.0),
                    snap: false,
                }),
                process_header,
                process_column
            ]
            .spacing(6),
        )
        .width(Length::FillPortion(2));

        let process_content = container(
            column![
                text("Monitor Processes").size(14).style(|_| text::Style {
                    color: Some(Color::from_rgb(0.8, 0.8, 0.8))
                }),
                rule::horizontal(2).style(|_| rule::Style {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    radius: 1.0.into(),
                    fill_mode: rule::FillMode::Percent(100.0),
                    snap: false,
                }),
                row![
                    selected_column,
                    rule::vertical(1).style(|_| rule::Style {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        radius: 1.0.into(),
                        fill_mode: rule::FillMode::Full,
                        snap: false,
                    }),
                    process_list_column
                ]
                .height(Length::Fill)
            ]
            .spacing(8),
        );

        // Assemble Sidebar
        let left_sidebar_content = row![
            column![toggle_btn]
                .width(Length::Fixed(30.0))
                .align_x(Center),
            container(process_content).width(Length::Fixed(520.0))
        ]
        .width(Length::Fixed(SIDEBAR_EXPANDED_WIDTH));

        let left_sidebar = container(left_sidebar_content)
            .width(Length::Fixed(current_sidebar_width))
            .height(Length::Fill)
            .style(styles::card_container_style)
            .padding(10);

        /*
        ========== TEMPERATURE SECTION ==========
        */
        let temp_section = column![
            row![text("Temperature").size(18).width(Length::Fill)].padding(5),
            container(
                self.temp_graph
                    .view()
                    .map(PlotWindowMessage::TempPlotMessage)
            )
            .height(Length::Fill)
            .width(Length::Fill)
            .style(styles::card_container_style),
        ]
        .spacing(10)
        .width(Length::FillPortion(2));

        /*
        ========== POWER/USAGE METRICS COLUMN ==========
        */
        let metrics_column = column![
            // CPU Power/Usage
            column![
                row![text("CPU Metrics").size(18).width(Length::Fill)].padding(5),
                container(
                    self.cpu_power_usage_graph
                        .view()
                        .map(PlotWindowMessage::CPUPowerUsagePlotMessage)
                )
                .height(Length::FillPortion(1))
                .width(Length::Fill)
                .style(styles::card_container_style),
            ]
            .spacing(10),
            text(" ").size(5),
            // GPU Power/Usage
            column![
                row![text("GPU Metrics").size(18).width(Length::Fill)].padding(5),
                container(
                    self.gpu_power_usage_graph
                        .view()
                        .map(PlotWindowMessage::GPUPowerUsagePlotMessage)
                )
                .height(Length::FillPortion(1))
                .width(Length::Fill)
                .style(styles::card_container_style),
            ]
            .spacing(10),
        ]
        .width(Length::FillPortion(3));

        /*
        ========== MAIN LAYOUT ==========
        */
        let content = row![
            left_sidebar,
            rule::vertical(1).style(|_theme| rule::Style {
                color: Color::from_rgb(0.25, 0.25, 0.25),
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: false,
            }),
            temp_section,
            rule::vertical(1).style(|_theme| rule::Style {
                color: Color::from_rgb(0.25, 0.25, 0.25),
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: false,
            }),
            metrics_column
        ]
        .spacing(15)
        .padding(15)
        .height(Length::Fill)
        .width(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.13))),
                ..Default::default()
            })
            .into()
    }

    fn process_column<'a>(
        sys: &'a HashMap<Pid, Process>,
    ) -> Column<'a, PlotWindowMessage, Theme, iced::Renderer> {
        Column::with_children(
            sys.iter()
                .map(|(_pid, process)| {
                    row![
                        text(process.name().to_string_lossy())
                            .size(11)
                            .width(Length::FillPortion(3)),
                        text(format!("{:.1}%", process.cpu_usage()))
                            .size(11)
                            .width(Length::Fixed(50.0)),
                        text(format!("{}K", process.memory() / 1024))
                            .size(11)
                            .width(Length::Fixed(55.0)),
                        button("+")
                            .padding([2, 5])
                            .style(compact_icon_button_style)
                            .on_press(PlotWindowMessage::ProcessSelected(
                                process.name().to_string_lossy().to_string()
                            )),
                        text("").width(Length::Fixed(10.0)), // Spacer for scrollbar
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect::<Vec<Element<'a, PlotWindowMessage, Theme, iced::Renderer>>>(),
        )
        .spacing(3)
    }
}
