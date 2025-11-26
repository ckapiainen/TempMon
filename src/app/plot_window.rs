use crate::app::cpu_total_power_and_usage_graph::PowerAndUsageGraph;
use crate::app::settings::TempUnits;
use crate::app::styles;
use crate::app::temp_graph::TemperatureGraph;
use crate::utils::csv_logger::CsvLogger;
use iced::widget::{button, column, combo_box, container, row, rule, scrollable, svg, text};
use iced::{window, Alignment, Color, Element, Length, Subscription}; // Added Subscription
use lilt::{Animated, Easing}; // Added lilt imports
use std::time::Instant; // Added Instant

const SIDEBAR_EXPANDED_WIDTH: f32 = 220.0;
const SIDEBAR_COLLAPSED_WIDTH: f32 = 50.0;

pub struct PlotWindow {
    temp_graph: TemperatureGraph,
    total_power_and_usage_graph: PowerAndUsageGraph,
    // Process monitoring
    available_processes: Vec<String>,
    selected_processes: Vec<String>,
    process_combo_box: combo_box::State<String>,
    // Process sidebar state
    sidebar_expanded: Animated<f32, Instant>,
    now: Instant,
}

#[derive(Debug, Clone)]
pub enum PlotWindowMessage {
    TempPlotMessage(iced_plot::PlotUiMessage),
    PowersAndUsagePlotMessage(iced_plot::PlotUiMessage),
    Tick,
    ToggleSidebar,
    ProcessSelected(String),
    RemoveProcess(String),
}

impl PlotWindow {
    pub fn new(temp_units_from_settings: String) -> Self {
        let units = if temp_units_from_settings == "Celsius" {
            "C"
        } else {
            "F"
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
            total_power_and_usage_graph: PowerAndUsageGraph::new(),
            process_combo_box: combo_box::State::new(available_processes.clone()),
            available_processes,
            selected_processes: vec![],
            sidebar_expanded: Animated::new(1.0).duration(300.0).easing(Easing::EaseInOut),
            now: Instant::now(),
        }
    }

    pub fn update(&mut self, csv_logger: &CsvLogger, message: PlotWindowMessage, units: TempUnits) {
        match message {
            PlotWindowMessage::TempPlotMessage(msg) => self.temp_graph.update_ui(msg),
            PlotWindowMessage::PowersAndUsagePlotMessage(msg) => {
                self.total_power_and_usage_graph.update_ui(msg)
            }
            PlotWindowMessage::Tick => {
                self.now = Instant::now();
                self.temp_graph.update_data(csv_logger, units);
                self.total_power_and_usage_graph.update_data(csv_logger);
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
                self.update_combo_box();
            }
            PlotWindowMessage::RemoveProcess(proc) => {
                self.selected_processes.retain(|p| p != &proc);
                self.update_combo_box();
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

    fn update_combo_box(&mut self) {
        let available: Vec<String> = self
            .available_processes
            .iter()
            .filter(|p| !self.selected_processes.contains(p))
            .cloned()
            .collect();
        self.process_combo_box = combo_box::State::new(available);
    }

    pub fn view(&self) -> Element<'_, PlotWindowMessage> {
        let sidebar_animation_factor = self
            .sidebar_expanded
            .animate(std::convert::identity, self.now);

        let current_sidebar_width = SIDEBAR_COLLAPSED_WIDTH
            + (sidebar_animation_factor * (SIDEBAR_EXPANDED_WIDTH - SIDEBAR_COLLAPSED_WIDTH));
        let is_collapsed = self.sidebar_expanded.value < 0.5;

        /*
         --- SIDEBAR CONTENT ---
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
            .align_x(iced::Center)
            .align_y(iced::Center)
            .width(30)
            .height(30),
        )
        .on_press(PlotWindowMessage::ToggleSidebar)
        .style(styles::ghost_icon_button_style)
        .padding(4);

        let process_content = container(
            column![
                text("Monitor Processes").size(15).style(|_| text::Style {
                    color: Some(Color::from_rgb(0.8, 0.8, 0.8))
                }),
                rule::horizontal(2).style(|_| rule::Style {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    radius: 1.0.into(),
                    fill_mode: rule::FillMode::Percent(100.0),
                    snap: false,
                }),
                scrollable(
                    column![
                        column![
                            text("Add Process").size(13),
                            combo_box(
                                &self.process_combo_box,
                                "Type to search...",
                                None,
                                PlotWindowMessage::ProcessSelected
                            )
                            .width(Length::Fill)
                        ]
                        .spacing(8),
                        rule::horizontal(1),
                        text("Selected:").size(13),
                        // Selected Pills
                        if self.selected_processes.is_empty() {
                            column![text("No processes selected")
                                .size(13)
                                .style(|_| text::Style {
                                    color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                                })]
                        } else {
                            column![row(self
                                .selected_processes
                                .iter()
                                .map(|proc| {
                                    button(
                                        row![text(proc).size(14), text(" ×").size(16)]
                                            .spacing(4)
                                            .align_y(Alignment::Center),
                                    )
                                    .on_press(PlotWindowMessage::RemoveProcess(proc.clone()))
                                    .style(styles::compact_icon_button_style)
                                    .padding([4, 10])
                                    .into()
                                })
                                .collect::<Vec<_>>())
                            .spacing(6)
                            .wrap()]
                        }
                    ]
                    .spacing(15)
                )
                .height(Length::Fill)
            ]
            .spacing(10),
        );

        // Assemble Sidebar
        let left_sidebar_content = row![
            column![toggle_btn]
                .width(Length::Fixed(50.0))
                .align_x(iced::Center), // Button column
            container(process_content)
                .padding(1)
                .width(Length::Fixed(150.0)) // Content column
        ]
        .width(Length::Fixed(SIDEBAR_EXPANDED_WIDTH));

        let left_sidebar = container(left_sidebar_content)
            .width(Length::Fixed(current_sidebar_width))
            .height(Length::Fill)
            .style(styles::card_container_style)
            .padding(5);

        /*
        --- CPU GRAPHS ---
        */
        let cpu_section = column![
            row![text("CPU Metrics").size(18).width(Length::Fill)].padding(5),
            container(
                self.temp_graph
                    .view()
                    .map(PlotWindowMessage::TempPlotMessage)
            )
            .height(Length::FillPortion(1))
            .width(Length::Fill)
            .style(styles::card_container_style),
            text(" ").size(5),
            container(
                self.total_power_and_usage_graph
                    .view()
                    .map(PlotWindowMessage::PowersAndUsagePlotMessage)
            )
            .height(Length::FillPortion(1))
            .width(Length::Fill)
            .style(styles::card_container_style),
        ]
        .spacing(10)
        .width(Length::FillPortion(3));

        /*
          --- GPU GRAPHS ---
        */
        let gpu_placeholder = container(
            column![
                text("GPU Metrics (Coming Soon)")
                    .size(16)
                    .style(|_theme| text::Style {
                        color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                    }),
                rule::horizontal(2),
            ]
            .spacing(15),
        )
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .padding(15)
        .style(styles::card_container_style);

        /*
        --- MAIN LAYOUT ---
        */
        let content = row![
            left_sidebar,
            rule::vertical(1).style(|_theme| rule::Style {
                color: Color::from_rgb(0.25, 0.25, 0.25),
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: false,
            }),
            cpu_section,
            rule::vertical(1).style(|_theme| rule::Style {
                color: Color::from_rgb(0.25, 0.25, 0.25),
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: false,
            }),
            gpu_placeholder
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
}
