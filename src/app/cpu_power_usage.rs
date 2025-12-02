use crate::utils::csv_logger::{ComponentType, CsvLogger};
use chrono::DateTime;
use iced::{Color, Element};
use iced_plot::{
    LineStyle, MarkerStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series, Tick, TickWeight,
    TooltipContext,
};

pub struct CPUPowerAndUsageGraph {
    widget: PlotWidget,
    first_timestamp: Option<i64>,
}

impl CPUPowerAndUsageGraph {
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

    pub fn update_data(&mut self, csv_logger: &CsvLogger) {
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

        // Extract power series
        let mut power_series: Vec<[f64; 2]> = buffer
            .iter()
            .filter(|entry| entry.component_type == ComponentType::CPU)
            .filter_map(|entry| {
                let ts = DateTime::parse_from_rfc3339(&entry.timestamp).ok()?;
                let x_seconds = (ts.timestamp() - start_ts) as f64;
                Some([x_seconds, entry.power_draw as f64])
            })
            .collect();

        // Extract usage series
        let mut usage_series: Vec<[f64; 2]> = buffer
            .iter()
            .filter(|entry| entry.component_type == ComponentType::CPU)
            .filter_map(|entry| {
                let ts = DateTime::parse_from_rfc3339(&entry.timestamp).ok()?;
                let x_seconds = (ts.timestamp() - start_ts) as f64;
                Some([x_seconds, entry.usage as f64])
            })
            .collect();

        if !power_series.is_empty() && !usage_series.is_empty() {
            let current_time = power_series.last().unwrap()[0];
            let window_size = 60.0;
            let right_padding = 12.0; // start rolling the graph 12 sec before the end
            let view_end = current_time + right_padding;

            // Scrolling logic
            if view_end > window_size {
                self.widget.set_x_lim(view_end - window_size, view_end);
            } else {
                self.widget.set_x_lim(0.0, window_size);
            }

            // Workaround: Pad to 33 points to force wgpu buffer update.
            // Necessary to display points between 0 and 33
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

            // Remove old series
            self.widget.remove_series("waiting for power/usage data");
            self.widget.remove_series("CPU Power (W)");
            self.widget.remove_series("CPU Usage (%)");

            // Add power series (orange/yellow color)
            let power = Series::new(
                power_series,
                MarkerStyle::circle(4.0),
                LineStyle::Solid { width: 4.0 },
            )
            .with_label("CPU Power (W)")
            .with_color(Color::from_rgb(1.0, 0.6, 0.0)); // Orange

            // Add usage series (blue/cyan color)
            let usage = Series::new(
                usage_series,
                MarkerStyle::circle(4.0),
                LineStyle::Solid { width: 4.0 },
            )
            .with_label("CPU Usage (%)")
            .with_color(Color::from_rgb(0.2, 0.6, 1.0)); // Blue

            self.widget.add_series(power).unwrap();
            self.widget.add_series(usage).unwrap();
        }
    }
}
