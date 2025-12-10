use crate::collectors::GpuData;
use crate::types::ComponentType;
use crate::utils::csv_logger::CsvLogger;
use chrono::DateTime;
use iced::{Color, Element};
use iced_plot::{
    LineStyle, MarkerStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series, Tick, TickWeight,
    TooltipContext,
};

pub struct GPUPowerAndUsageGraph {
    widget: PlotWidget,
    first_timestamp: Option<i64>,
}

impl GPUPowerAndUsageGraph {
    pub fn new() -> Self {
        // Initial dummy series
        let dummy_series =
            Series::circles(vec![[0.0, 0.0]], 3.0).with_label("waiting for power/usage data");

        Self {
            widget: PlotWidgetBuilder::new()
                .with_x_label("Time (s)")
                .with_tooltips(true)
                .with_tooltip_provider(|ctx: &TooltipContext| {
                    format!("Time: {:.0}s\nValue: {:.1}", ctx.x, ctx.y)
                })
                .with_autoscale_on_updates(true)
                .with_x_lim(0.0, 60.0)
                .with_y_lim(0.0, 150.0)
                .with_y_tick_producer(|min, max| {
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
                .with_tick_label_size(10.0)
                .with_crosshairs(true)
                .with_cursor_provider(|x, y| format!("Time: {:.0}s\nValue: {:.1}", x, y))
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

    pub fn update_data(&mut self, csv_logger: &CsvLogger, gpu_data: &[GpuData]) {
        let buffer = &csv_logger.graph_data_buffer;
        if buffer.is_empty() {
            return;
        }

        // Try to determine the baseline timestamp (t=0)
        if self.first_timestamp.is_none() {
            if let Ok(t) = DateTime::parse_from_rfc3339(&buffer[0].timestamp) {
                println!("First timestamp: {}", t.timestamp());
                self.first_timestamp = Some(t.timestamp());
            }
        }
        let start_ts = self.first_timestamp.unwrap_or(0);

        // Color palettes for multiple GPUs
        const POWER_COLORS: [Color; 4] = [
            Color::from_rgb(1.0, 0.5, 0.0),   // Orange - GPU 0
            Color::from_rgb(1.0, 0.2, 0.2),   // Red - GPU 1
            Color::from_rgb(1.0, 0.8, 0.0),   // Yellow - GPU 2
            Color::from_rgb(1.0, 0.0, 0.5),   // Magenta - GPU 3
        ];

        const USAGE_COLORS: [Color; 4] = [
            Color::from_rgb(0.0, 0.5, 1.0),   // Blue - GPU 0
            Color::from_rgb(0.0, 0.8, 0.8),   // Cyan - GPU 1
            Color::from_rgb(0.0, 1.0, 0.3),   // Green - GPU 2
            Color::from_rgb(0.5, 0.0, 1.0),   // Purple - GPU 3
        ];

        // Collect all GPU entries
        let gpu_entries: Vec<&_> = buffer
            .iter()
            .filter(|entry| entry.component_type == ComponentType::GPU)
            .collect();

        if gpu_entries.is_empty() || gpu_data.is_empty() {
            return;
        }

        // Remove old series
        self.widget.remove_series("waiting for power/usage data");
        for gpu in gpu_data.iter() {
            self.widget.remove_series(&format!("{} Power (W)", gpu.name));
            self.widget.remove_series(&format!("{} Usage (%)", gpu.name));
        }

        let mut any_series_added = false;
        let mut latest_time: f64 = 0.0;

        // Create separate series for each GPU
        for (gpu_idx, gpu) in gpu_data.iter().enumerate() {
            // Extract power series for this GPU (match by position in log cycle)
            let mut power_series: Vec<[f64; 2]> = gpu_entries
                .iter()
                .enumerate()
                .filter(|(idx, _)| idx % gpu_data.len() == gpu_idx)
                .filter_map(|(_, entry)| {
                    let ts = DateTime::parse_from_rfc3339(&entry.timestamp).ok()?;
                    let x_seconds = (ts.timestamp() - start_ts) as f64;
                    Some([x_seconds, entry.power_draw as f64])
                })
                .collect();

            // Extract usage series for this GPU
            let mut usage_series: Vec<[f64; 2]> = gpu_entries
                .iter()
                .enumerate()
                .filter(|(idx, _)| idx % gpu_data.len() == gpu_idx)
                .filter_map(|(_, entry)| {
                    let ts = DateTime::parse_from_rfc3339(&entry.timestamp).ok()?;
                    let x_seconds = (ts.timestamp() - start_ts) as f64;
                    Some([x_seconds, entry.usage as f64])
                })
                .collect();

            if !power_series.is_empty() && !usage_series.is_empty() {
                any_series_added = true;
                latest_time = latest_time.max(power_series.last().unwrap()[0]);

                // Workaround: Pad to 33 points to force wgpu buffer update
                if power_series.len() < 33 {
                    let last_point = *power_series.last().unwrap();
                    while power_series.len() < 33 {
                        power_series.push(last_point);
                    }
                }

                if usage_series.len() < 33 {
                    let last_point = *usage_series.last().unwrap();
                    while usage_series.len() < 33 {
                        usage_series.push(last_point);
                    }
                }

                // Add power series for this GPU
                let power = Series::new(
                    power_series,
                    MarkerStyle::circle(4.0),
                    LineStyle::Solid { width: 4.0 },
                )
                .with_label(&format!("{} Power (W)", gpu.name))
                .with_color(POWER_COLORS[gpu_idx % POWER_COLORS.len()]);

                // Add usage series for this GPU
                let usage = Series::new(
                    usage_series,
                    MarkerStyle::circle(4.0),
                    LineStyle::Solid { width: 4.0 },
                )
                .with_label(&format!("{} Usage (%)", gpu.name))
                .with_color(USAGE_COLORS[gpu_idx % USAGE_COLORS.len()]);

                self.widget.add_series(power).unwrap();
                self.widget.add_series(usage).unwrap();
            }
        }

        // Update scrolling based on latest time
        if any_series_added {
            let window_size = 60.0;
            let right_padding = 12.0;
            let view_end = latest_time + right_padding;

            if view_end > window_size {
                self.widget.set_x_lim(view_end - window_size, view_end);
            } else {
                self.widget.set_x_lim(0.0, window_size);
            }
        }
    }
}
