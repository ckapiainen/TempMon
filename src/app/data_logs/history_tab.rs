use super::file_list;
use super::metadata::LogFileMetadata;
use crate::utils::csv_logger::CsvLogger;
use iced::widget::{container, text};
use iced::{Color, Element, Length};
use std::path::PathBuf;

pub struct HistoricalTab {
    pub log_files: Vec<LogFileMetadata>,
    pub selected_file: Option<PathBuf>,
    pub show_only_process_logs: bool,
}

#[derive(Debug, Clone)]
pub enum HistoricalMessage {
    LoadFiles,
    FileSelected(PathBuf),
    ToggleProcessFilter(bool),
}

impl HistoricalTab {
    pub fn new() -> Self {
        Self {
            log_files: Vec::new(),
            selected_file: None,
            show_only_process_logs: false,
        }
    }

    pub fn update(&mut self, message: HistoricalMessage, csv_logger: &CsvLogger) {
        match message {
            HistoricalMessage::LoadFiles => {
                self.load_files(csv_logger);
            }
            HistoricalMessage::FileSelected(path) => {
                self.selected_file = Some(path.clone());
                // TODO: In future, load the file data into graphs
            }
            HistoricalMessage::ToggleProcessFilter(enabled) => {
                self.show_only_process_logs = enabled;
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

        // Selected file info graph
        let info_panel = if let Some(path) = &self.selected_file {
            container(
                text(format!(
                    "Selected: {}\n\n",
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                ))
                .size(16)
                .style(|_| text::Style {
                    color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                }),
            )
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(crate::app::styles::card_container_style)
        } else {
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
