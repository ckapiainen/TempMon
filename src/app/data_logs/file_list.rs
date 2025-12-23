use super::metadata::LogFileMetadata;
use crate::app::styles;
use iced::widget::{button, column, container, row, rule, scrollable, text, Column};
use iced::{Alignment, Color, Element, Length};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileListMessage {
    FileSelected(PathBuf),
    ToggleProcessFilter(bool),
}

pub fn view<'a, Message>(
    files: &'a [LogFileMetadata],
    selected_file: &'a Option<PathBuf>,
    show_only_process_logs: bool,
    message_mapper: impl Fn(FileListMessage) -> Message + 'a + Copy,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    // Header with process filter toggle
    let filter_btn_text = if show_only_process_logs {
        "Process Logs ✓"
    } else {
        "All Logs"
    };

    let header = row![
        text("Log Files").size(18),
        container(
            button(text(filter_btn_text).size(12))
                .on_press(message_mapper(FileListMessage::ToggleProcessFilter(
                    !show_only_process_logs
                )))
                .style(styles::rounded_button_style)
                .padding([4, 8])
        )
        .align_x(Alignment::End)
        .width(Length::Fill)
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    // Filter files based on toggle
    let filtered_files: Vec<_> = if show_only_process_logs {
        files.iter().filter(|f| f.has_process_data).collect()
    } else {
        files.iter().collect()
    };

    // File list header (column labels)
    let list_header = row![
        text("Date")
            .size(11)
            .width(Length::FillPortion(3))
            .style(|_| text::Style {
                color: Some(Color::from_rgb(0.7, 0.7, 0.7))
            }),
        text("Entries")
            .size(11)
            .width(Length::Fixed(60.0))
            .style(|_| text::Style {
                color: Some(Color::from_rgb(0.7, 0.7, 0.7))
            }),
        text("Size")
            .size(11)
            .width(Length::Fixed(60.0))
            .style(|_| text::Style {
                color: Some(Color::from_rgb(0.7, 0.7, 0.7))
            }),
        text("Proc")
            .size(11)
            .width(Length::Fixed(40.0))
            .style(|_| text::Style {
                color: Some(Color::from_rgb(0.7, 0.7, 0.7))
            }),
    ]
    .spacing(8)
    .padding([0, 10]);

    // File rows
    let file_rows = Column::with_children(
        filtered_files
            .iter()
            .map(|file_meta| {
                let is_selected = selected_file
                    .as_ref()
                    .map_or(false, |p| p == &file_meta.path);

                let row_style = if is_selected {
                    styles::selected_row_style
                } else {
                    styles::file_row_style
                };

                button(
                    row![
                        text(&file_meta.date).size(12).width(Length::FillPortion(3)),
                        text(format!("{}", file_meta.entry_count))
                            .size(12)
                            .width(Length::Fixed(60.0)),
                        text(file_meta.format_size())
                            .size(12)
                            .width(Length::Fixed(60.0)),
                        text(if file_meta.has_process_data {
                            "✓"
                        } else {
                            ""
                        })
                        .size(12)
                        .width(Length::Fixed(40.0)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .padding([4, 10]),
                )
                .on_press(message_mapper(FileListMessage::FileSelected(
                    file_meta.path.clone(),
                )))
                .style(row_style)
                .width(Length::Fill)
                .into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(4);

    let scrollable_list = scrollable(
        column![
            header,
            rule::horizontal(1).style(|_| rule::Style {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                radius: 1.0.into(),
                fill_mode: rule::FillMode::Percent(100.0),
                snap: false,
            }),
            list_header,
            file_rows
        ]
        .spacing(8),
    )
    .style(styles::sleek_scrollbar_style)
    .height(Length::Fill);

    container(scrollable_list)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(styles::card_container_style)
        .padding(10)
        .into()
}
