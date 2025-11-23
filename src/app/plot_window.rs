use crate::app::settings::TempUnits;
use crate::utils::csv_logger::CsvLogger;
use chrono::DateTime;
use iced::{Color, Element};
use iced_plot::{LineStyle, MarkerStyle, PlotWidget, PlotWidgetBuilder, Series};

pub struct PlotWindow {
    plot: PlotWidget,
    // We store the timestamp of the first point we ever see
    // to normalize the X-axis (start at t=0s)
    first_timestamp: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum PlotWindowMessage {
    PlotUiMessage(iced_plot::PlotUiMessage),
    Tick,
}

impl PlotWindow {
    pub fn new() -> Self {
        // Initial dummy series
        let dummy_series = Series::circles(vec![[0.0, 0.0]], 3.0).with_label("waiting for data");

        Self {
            plot: PlotWidgetBuilder::new()
                .with_y_label("Temperature (°C)")
                .with_x_label("Time (s)")
                .with_tooltips(true)
                .with_autoscale_on_updates(true)
                .with_x_lim(0.0, 60.0)
                .with_y_lim(0.0, 100.0)
                .add_series(dummy_series)
                .build()
                .unwrap(),
            first_timestamp: None,
        }
    }

    pub fn update(&mut self, csv_logger: &CsvLogger, message: PlotWindowMessage, units: TempUnits) {
        match message {
            PlotWindowMessage::PlotUiMessage(msg) => self.plot.update(msg),
            PlotWindowMessage::Tick => {
                let label = match units {
                    TempUnits::Celsius => "Temperature (°C)",
                    TempUnits::Fahrenheit => "Temperature (°F)",
                };
                // Set y label and limits based on selected units
                self.plot.set_y_axis_label(label);
                match units {
                    TempUnits::Celsius => self.plot.set_y_lim(0.0, 100.0),
                    TempUnits::Fahrenheit => self.plot.set_y_lim(32.0, 212.0),
                }

                let buffer = &csv_logger.graph_data_buffer;

                if !buffer.is_empty() {
                    // Try to determine the baseline timestamp (t=0)
                    if self.first_timestamp.is_none() {
                        if let Ok(t) = DateTime::parse_from_rfc3339(&buffer[0].timestamp) {
                            println!("First timestamp: {}", t.timestamp());
                            self.first_timestamp = Some(t.timestamp());
                        }
                    }
                    let start_ts = self.first_timestamp.unwrap_or(0);

                    let mut cpu_temp_series: Vec<[f64; 2]> = buffer
                        .iter()
                        .filter_map(|entry| {
                            // Parse timestamp
                            let ts = DateTime::parse_from_rfc3339(&entry.timestamp).ok()?;
                            let x_seconds = (ts.timestamp() - start_ts) as f64;

                            // Convert temperature
                            let y_temp = units.convert(entry.temperature, TempUnits::Celsius);

                            Some([x_seconds, y_temp as f64])
                        })
                        .collect();

                    if !cpu_temp_series.is_empty() {
                        let current_time = cpu_temp_series.last().unwrap()[0];
                        let window_size = 60.0;

                        // Shift the camera to follow the latest data
                        if current_time > window_size {
                            self.plot
                                .set_x_lim(current_time - window_size, current_time);
                        } else {
                            self.plot.set_x_lim(0.0, window_size);
                        }

                        // If fewer than 33 points duplicate the last point
                        // Workaround: Pad to 33 points to force wgpu buffer update.
                        // Necessary to display points between 0 and 33
                        if cpu_temp_series.len() < 33 {
                            let last_point = *cpu_temp_series.last().unwrap();
                            while cpu_temp_series.len() < 33 {
                                cpu_temp_series.push(last_point);
                            }
                        }

                        self.plot.remove_series("waiting for data");
                        self.plot.remove_series("CPU Temperature");

                        let temp_series = Series::new(
                            cpu_temp_series,
                            MarkerStyle::circle(5.0),
                            LineStyle::Solid,
                        )
                        .with_label("CPU Temperature")
                        .with_color(Color::from_rgb(1.0, 0.2, 0.2));

                        self.plot.add_series(temp_series).unwrap();
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, PlotWindowMessage> {
        self.plot.view().map(PlotWindowMessage::PlotUiMessage)
    }
}
