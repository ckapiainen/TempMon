use crate::app::cpu_total_power_and_usage_graph::PowerAndUsageGraph;
use crate::app::settings::TempUnits;
use crate::app::temp_graph::TemperatureGraph;
use crate::utils::csv_logger::CsvLogger;
use iced::widget::{column, container, row};
use iced::Element;

pub struct PlotWindow {
    temp_graph: TemperatureGraph, // for gpu and cpu
    total_power_and_usage_graph: PowerAndUsageGraph,
}

#[derive(Debug, Clone)]
pub enum PlotWindowMessage {
    TempPlotMessage(iced_plot::PlotUiMessage),
    PowersAndUsagePlotMessage(iced_plot::PlotUiMessage),
    Tick,
}
// TODO: Plotting to other values. Stop forced scrolling when user drags the plot. Add min/max values: horizontal lines?
// TODO: Separate refresh reate for graphs. Somehow handle high refresh rates >=1s marker frequency. Currently they are clumped together.
// TODO: Fahrenheit is completely broken. Larger y-axis tick interval for fahrenheit.
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
        }
    }

    pub fn view(&self) -> Element<'_, PlotWindowMessage> {
        let graph_row = row![
            self.temp_graph
                .view()
                .map(PlotWindowMessage::TempPlotMessage),
            self.total_power_and_usage_graph
                .view()
                .map(PlotWindowMessage::PowersAndUsagePlotMessage)
        ]
        .width(800)
        .height(400);
        let content_column = column![graph_row];
        container(content_column).padding(10).into()
    }
}
