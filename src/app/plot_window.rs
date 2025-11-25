use crate::app::cpu_total_power_and_usage_graph::PowerAndUsageGraph;
use crate::app::settings::TempUnits;
use crate::app::styles; // Assuming you have shared styles
use crate::app::temp_graph::TemperatureGraph;
use crate::utils::csv_logger::CsvLogger;
use iced::widget::{button, checkbox, column, container, row, rule, scrollable, text};
use iced::{Alignment, Color, Element, Length};

pub struct PlotWindow {
    temp_graph: TemperatureGraph,
    total_power_and_usage_graph: PowerAndUsageGraph,
    selected_processes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PlotWindowMessage {
    TempPlotMessage(iced_plot::PlotUiMessage),
    PowersAndUsagePlotMessage(iced_plot::PlotUiMessage),
    Tick,
    ProcessToggled(String, bool),
}

impl PlotWindow {
    pub fn new(temp_units_from_settings: String) -> Self {
        let units = if temp_units_from_settings == "Celsius" {
            "C"
        } else {
            "F"
        };

        Self {
            temp_graph: TemperatureGraph::new(units),
            total_power_and_usage_graph: PowerAndUsageGraph::new(),
            // Mock data for the list
            selected_processes: vec![],
        }
    }

    pub fn update(&mut self, csv_logger: &CsvLogger, message: PlotWindowMessage, units: TempUnits) {
        match message {
            PlotWindowMessage::TempPlotMessage(msg) => self.temp_graph.update_ui(msg),
            PlotWindowMessage::PowersAndUsagePlotMessage(msg) => {
                self.total_power_and_usage_graph.update_ui(msg)
            }
            PlotWindowMessage::Tick => {
                self.temp_graph.update_data(csv_logger, units);
                self.total_power_and_usage_graph.update_data(csv_logger);
            }
            PlotWindowMessage::ProcessToggled(_proc, _state) => {
                // TODO: Handle process selection logic
            }
        }
    }

    pub fn view(&self) -> Element<'_, PlotWindowMessage> {
        /*
        Mock list of processes for now
        */
        const PROCESSES: &[&str] = &[
            "chrome.exe",
            "rustc.exe",
            "code.exe",
            "spotify.exe",
            "system",
        ];

        let process_list = column(
            PROCESSES
                .iter()
                .map(|p| {
                    row![
                        // Using text for now, checkbox needs state handling
                        text(*p).size(14).width(Length::Fill),
                        checkbox("", false)
                            .on_toggle(|_| PlotWindowMessage::ProcessToggled(p.to_string(), false))
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(10);

        let left_sidebar = container(
            column![
                text("Monitored Processes")
                    .size(16)
                    .style(|_theme| text::Style {
                        color: Some(Color::from_rgb(0.8, 0.8, 0.8))
                    }),
                rule::horizontal(2).style(|_theme| rule::Style {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    radius: 1.0.into(),
                    fill_mode: rule::FillMode::Percent(100.0),
                    snap: false,
                }),
                scrollable(process_list).height(Length::Fill)
            ]
            .spacing(15),
        )
        .width(Length::Fixed(220.0))
        .height(Length::Fill)
        .padding(15)
        .style(styles::card_container_style);

        /*
        MIDDLE PANEL: CPU GRAPHS
        */
        let cpu_section = column![
            // Header
            row![
                text("CPU Metrics").size(18).width(Length::Fill),
                // Optional: CPU stats summary here later
            ]
            .padding(5),
            // Temperature Graph
            container(
                self.temp_graph
                    .view()
                    .map(PlotWindowMessage::TempPlotMessage)
            )
            .height(Length::FillPortion(1))
            .width(Length::Fill)
            .style(styles::card_container_style),
            // Spacing
            text(" ").size(5),
            // Power/Usage Graph
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
        .width(Length::FillPortion(3)); // Takes up 3x space relative to other flexible columns

        /*
        RIGHT PANEL: FUTURE GPU / STATS
        */
        let gpu_placeholder = container(
            column![
                text("GPU Metrics (Coming Soon)")
                    .size(16)
                    .style(|_theme| text::Style {
                        color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                    }),
                rule::horizontal(2),
                // Empty space for now
            ]
            .spacing(15),
        )
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .padding(15)
        .style(styles::card_container_style);

        // --- MAIN LAYOUT ASSEMBLY ---
        let content = row![
            left_sidebar,
            // Vertical Rule Separator
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
                background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.13))), // Dark background
                ..Default::default()
            })
            .into()
    }
}
