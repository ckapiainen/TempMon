use super::file_list;
use super::metadata::LogFileMetadata;
use crate::app::data_logs::history_graphs::{CPUDataLog, GPUDataLog};
use crate::types::HardwareLogEntry;
use crate::utils::csv_logger::CsvLogger;
use iced::widget::{column, container, text};
use iced::{Color, Element, Length, Task};
use std::path::PathBuf;

pub struct HistoricalTab {
    pub log_files: Vec<LogFileMetadata>,
    pub selected_file: Option<PathBuf>,
    pub show_only_process_logs: bool,
    cpu_graph: Option<CPUDataLog>,
    gpu_graph: Option<GPUDataLog>,
}

#[derive(Debug, Clone)]
pub enum HistoricalMessage {
    LoadFiles,
    FileSelected(PathBuf),
    CreateGraphs { cpu_data: Vec<HardwareLogEntry>, gpu_data: Vec<HardwareLogEntry> },
    ToggleProcessFilter(bool),
    CPUPlotMessage(iced_plot::PlotUiMessage),
    GPUPlotMessage(iced_plot::PlotUiMessage),
}

impl HistoricalTab {
    pub fn new() -> Self {
        Self {
            log_files: Vec::new(),
            selected_file: None,
            show_only_process_logs: false,
            cpu_graph: None,
            gpu_graph: None,
        }
    }

    pub fn update(
        &mut self,
        message: HistoricalMessage,
        csv_logger: &CsvLogger,
    ) -> Task<HistoricalMessage> {
        match message {
            HistoricalMessage::LoadFiles => {
                self.load_files(csv_logger);
                Task::none()
            }
            HistoricalMessage::FileSelected(path) => {
                self.selected_file = Some(path.clone());

                // Destroy the old graphs first
                self.cpu_graph = None;
                self.gpu_graph = None;

                // Read the data
                let result = csv_logger.read(path.to_str().unwrap().to_string());
                let mut cpu_data = Vec::new();
                let mut gpu_data = Vec::new();
                if let Ok(entries) = result {
                    for entry in entries {
                        match entry.component_type {
                            crate::types::ComponentType::CPU => cpu_data.push(entry),
                            crate::types::ComponentType::GPU => gpu_data.push(entry),
                            _ => {} // Ignore RAM, SSD, and other types
                        }
                    }
                }

                Task::done(HistoricalMessage::CreateGraphs { cpu_data, gpu_data })
            }
            HistoricalMessage::CreateGraphs { cpu_data, gpu_data } => {
                if !cpu_data.is_empty() {
                    self.cpu_graph = Some(CPUDataLog::new(cpu_data));
                }
                if !gpu_data.is_empty() {
                    self.gpu_graph = Some(GPUDataLog::new(gpu_data));
                }
                Task::none()
            }
            HistoricalMessage::ToggleProcessFilter(enabled) => {
                self.show_only_process_logs = enabled;
                Task::none()
            }
            HistoricalMessage::CPUPlotMessage(msg) => {
                if let Some(graph) = &mut self.cpu_graph {
                    graph.update_ui(msg);
                }
                Task::none()
            }
            HistoricalMessage::GPUPlotMessage(msg) => {
                if let Some(graph) = &mut self.gpu_graph {
                    graph.update_ui(msg);
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, HistoricalMessage> {
        // File list panel
        let file_list_panel = file_list::view(
            &self.log_files,
            &self.selected_file,
            self.show_only_process_logs,
            |msg| match msg {
                file_list::FileListMessage::FileSelected(path) => {
                    HistoricalMessage::FileSelected(path)
                }
                file_list::FileListMessage::ToggleProcessFilter(enabled) => {
                    HistoricalMessage::ToggleProcessFilter(enabled)
                }
            },
        );

        // Selected file info/graph panel
        let info_panel = if let Some(path) = &self.selected_file {
            let has_cpu = self.cpu_graph.is_some();
            let has_gpu = self.gpu_graph.is_some();

            if has_cpu || has_gpu {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");

                let mut graphs_column = column![].spacing(15);

                // Add CPU graph if available
                if let Some(cpu_graph) = &self.cpu_graph {
                    graphs_column = graphs_column.push(
                        container(
                            column![
                                text(format!("CPU History - {}", filename)).size(16),
                                cpu_graph.view().map(HistoricalMessage::CPUPlotMessage)
                            ]
                            .spacing(10),
                        )
                        .width(Length::Fill)
                        .height(Length::FillPortion(1))
                        .padding(15)
                        .style(crate::app::styles::card_container_style),
                    );
                }

                // Add GPU graph if available
                if let Some(gpu_graph) = &self.gpu_graph {
                    graphs_column = graphs_column.push(
                        container(
                            column![
                                text(format!("GPU History - {}", filename)).size(16),
                                gpu_graph.view().map(HistoricalMessage::GPUPlotMessage)
                            ]
                            .spacing(10),
                        )
                        .width(Length::Fill)
                        .height(Length::FillPortion(1))
                        .padding(15)
                        .style(crate::app::styles::card_container_style),
                    );
                }

                container(graphs_column)
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
            } else {
                // No data message
                container(text("No data in selected file").size(16).style(|_| {
                    text::Style {
                        color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                    }
                }))
                .width(Length::FillPortion(2))
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(crate::app::styles::card_container_style)
            }
        } else {
            // No file selected message
            container(
                text("Select a log file from the list")
                    .size(16)
                    .style(|_| text::Style {
                        color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
                    }),
            )
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(crate::app::styles::card_container_style)
        };

        // Two-panel layout
        let content = iced::widget::row![file_list_panel, info_panel]
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

    fn load_files(&mut self, csv_logger: &CsvLogger) {
        match csv_logger.list_logs_files() {
            Ok(paths) => {
                self.log_files = paths
                    .into_iter()
                    .filter_map(LogFileMetadata::from_path)
                    .collect();

                // Sort by date descending (newest first)
                self.log_files.sort_by(|a, b| b.date.cmp(&a.date));
            }
            Err(e) => {
                eprintln!("Failed to load log files: {}", e);
            }
        }
    }
}
