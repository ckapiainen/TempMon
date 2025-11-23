use crate::app::settings::TempUnits;
use crate::app::termperature_graph::TemperatureGraph;
use crate::utils::csv_logger::CsvLogger;
use iced::widget::{container, row};
use iced::Element;

pub struct PlotWindow {
    temp_graph: TemperatureGraph, // for gpu and cpu
}

#[derive(Debug, Clone)]
pub enum PlotWindowMessage {
    PlotUiMessage(iced_plot::PlotUiMessage),
    Tick,
}
// TODO: Plotting to other values. Stop forced scrolling when user drags the plot. Add min/max values: horizontal lines?
impl PlotWindow {
    pub fn new(temp_units_from_settings: String) -> Self {
        let units = if temp_units_from_settings == "Celsius" {
            "C"
        } else {
            "F"
        };

        Self {
            temp_graph: TemperatureGraph::new(units),
        }
    }

    pub fn update(&mut self, csv_logger: &CsvLogger, message: PlotWindowMessage, units: TempUnits) {
        match message {
            PlotWindowMessage::PlotUiMessage(msg) => self.temp_graph.update_ui(msg),
            PlotWindowMessage::Tick => {
                self.temp_graph.update_data(csv_logger, units);
            }
        }
    }

    pub fn view(&self) -> Element<'_, PlotWindowMessage> {
        let graph_row = row![self.temp_graph.view().map(PlotWindowMessage::PlotUiMessage)];
        container(graph_row).width(550).height(550).into()
    }
}
