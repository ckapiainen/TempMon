use crate::app::settings::TempUnits;
use crate::collectors::GpuData;
use crate::utils::csv_logger::{ComponentType, CsvLogger};
use chrono::DateTime;
use iced::{Color, Element};
use iced_plot::{
    LineStyle, MarkerStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series, Tick, TickWeight,
    TooltipContext,
};
use std::sync::Arc;

pub struct TemperatureGraph {
    widget: PlotWidget,
    first_timestamp: Option<i64>,
}

impl TemperatureGraph {
    pub fn new(temp_units_from_settings: TempUnits) -> Self {
        let units = if temp_units_from_settings == TempUnits::Celsius {
            "C"
        } else {
            "F"
        };
        // Initial dummy series
        let dummy_series = Series::circles(vec![[0.0, 0.0]], 3.0).with_label("waiting for data");

        Self {
            widget: PlotWidgetBuilder::new()
                .with_x_label("Time (s)")
                .with_tooltips(true)
                .with_tooltip_provider(|ctx: &TooltipContext| {
                    format!("t: {:.0} s\nTemperature: {:.1}", ctx.x, ctx.y)
                })
                .with_autoscale_on_updates(true)
                .with_x_lim(0.0, 60.0)
                .with_y_lim(20.0, 100.0)
                .with_y_tick_producer(|min, max| {
                    let tick_interval = 15.0;
                    let start = (min / tick_interval).floor() * tick_interval;
                    let mut ticks = Vec::new();
                    let mut value = start;

                    while value <= max {
                        if value >= min {
                            ticks.push(Tick {
                                value,
                                step_size: tick_interval,
                                line_type: TickWeight::Major,
                            });
                        }
                        value += tick_interval;
                    }

                    ticks
                })
                .with_x_tick_producer(|min, max| {
                    let tick_interval = 25.0;
                    let start = (min / tick_interval).floor() * tick_interval;
                    let mut ticks = Vec::new();
                    let mut value = start;

                    while value <= max {
                        if value >= min {
                            ticks.push(Tick {
                                value,
                                step_size: tick_interval,
                                line_type: TickWeight::Major,
                            });
                        }
                        value += tick_interval;
                    }

                    ticks
                })
                .with_y_tick_formatter(|tick| format!("{:.1}", tick.value))
                .with_crosshairs(true)
                .with_cursor_provider(move |x, y| {
                    format!("Time: {:.0}\nTemp: {:.1}°{}", x, y, units)
                })
                .with_tick_label_size(12.0)
                .add_series(dummy_series)
                .build()
                .unwrap(),
            first_timestamp: None,
        }
    }

    pub fn view(&self) -> Element<'_, PlotUiMessage> {
        self.widget.view()
    }

    pub fn update_ui(&mut self, msg: PlotUiMessage) {
        self.widget.update(msg);
    }

    pub fn update_data(&mut self, csv_logger: &CsvLogger, units: TempUnits, gpu_data: &[GpuData]) {
        let buffer = &csv_logger.graph_data_buffer;
        if buffer.is_empty() {
            return;
        }

        // Update cursor provider with current units
        let unit_symbol = match units {
            TempUnits::Celsius => "C",
            TempUnits::Fahrenheit => "F",
        };
        self.widget.set_cursor_provider(Arc::new(move |x, y| {
            format!("Time: {:.0} s\nTemp: {:.1}°{}", x, y, unit_symbol)
        }));

        // Update Y-axis limits based on units
        match units {
            TempUnits::Celsius => self.widget.set_y_lim(20.0, 100.0),
            TempUnits::Fahrenheit => self.widget.set_y_lim(32.0, 212.0),
        }

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
            .filter(|entry| entry.component_type == ComponentType::CPU)
            .filter_map(|entry| {
                // Parse timestamp
                let ts = DateTime::parse_from_rfc3339(&entry.timestamp).ok()?;
                let x_seconds = (ts.timestamp() - start_ts) as f64;

                Some([x_seconds, entry.temperature as f64]) // NOTE: temp gets converted in main tempmon update loop
            })
            .collect();

        // Color palette for multiple GPUs
        const GPU_TEMP_COLORS: [Color; 4] = [
            Color::from_rgb(1.0, 0.5, 0.0), // Orange - GPU 0
            Color::from_rgb(1.0, 0.2, 0.2), // Red - GPU 1
            Color::from_rgb(1.0, 0.8, 0.0), // Yellow - GPU 2
            Color::from_rgb(1.0, 0.0, 0.5), // Magenta - GPU 3
        ];

        // Collect all GPU temp entries
        let gpu_entries: Vec<&_> = buffer
            .iter()
            .filter(|entry| entry.component_type == ComponentType::GPU)
            .collect();

        if !cpu_temp_series.is_empty() {
            let current_time = cpu_temp_series.last().unwrap()[0];
            let window_size = 60.0;
            let right_padding = 12.0;
            let view_end = current_time + right_padding;

            // Scrolling logic
            if view_end > window_size {
                self.widget.set_x_lim(view_end - window_size, view_end);
            } else {
                self.widget.set_x_lim(0.0, window_size);
            }

            // Pad CPU series if needed
            if cpu_temp_series.len() < 33 {
                let last_point = *cpu_temp_series.last().unwrap();
                while cpu_temp_series.len() < 33 {
                    cpu_temp_series.push(last_point);
                }
            }

            // Remove old series
            self.widget.remove_series("waiting for data");
            self.widget.remove_series("CPU Temperature");
            for gpu in gpu_data.iter() {
                self.widget.remove_series(&format!("{} Temp", gpu.name));
            }

            // Add CPU series
            let cpu_series = Series::new(
                cpu_temp_series,
                MarkerStyle::circle(4.0),
                LineStyle::Solid { width: 3.0 },
            )
            .with_label("CPU Temperature")
            .with_color(Color::from_rgb(0.2, 0.6, 1.0)); // Blue for CPU

            self.widget.add_series(cpu_series).unwrap();

            // Add separate series for each GPU
            if !gpu_entries.is_empty() && !gpu_data.is_empty() {
                for (gpu_idx, gpu) in gpu_data.iter().enumerate() {
                    // Extract temp series for this GPU (match by position in log cycle)
                    let mut gpu_temp_series: Vec<[f64; 2]> = gpu_entries
                        .iter()
                        .enumerate()
                        .filter(|(idx, _)| idx % gpu_data.len() == gpu_idx)
                        .filter_map(|(_, entry)| {
                            let ts = DateTime::parse_from_rfc3339(&entry.timestamp).ok()?;
                            let x_seconds = (ts.timestamp() - start_ts) as f64;
                            Some([x_seconds, entry.temperature as f64])
                        })
                        .collect();

                    if !gpu_temp_series.is_empty() {
                        // Pad if needed
                        if gpu_temp_series.len() < 33 {
                            let last_point = *gpu_temp_series.last().unwrap();
                            while gpu_temp_series.len() < 33 {
                                gpu_temp_series.push(last_point);
                            }
                        }

                        // Add GPU temp series
                        let gpu_series = Series::new(
                            gpu_temp_series,
                            MarkerStyle::circle(4.0),
                            LineStyle::Solid { width: 3.0 },
                        )
                        .with_label(&format!("{} Temp", gpu.name))
                        .with_color(GPU_TEMP_COLORS[gpu_idx % GPU_TEMP_COLORS.len()]);

                        self.widget.add_series(gpu_series).unwrap();
                    }
                }
            }
        }
    }
}
