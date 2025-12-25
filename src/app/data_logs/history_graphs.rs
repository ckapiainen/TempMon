use crate::types::HardwareLogEntry;
use chrono::DateTime;
use iced::{Color, Element};
use iced_plot::{
    LineStyle, MarkerStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series, Tick, TickWeight,
    TooltipContext,
};
use std::collections::HashMap;

pub struct GPUDataLog {
    widget: PlotWidget,
}

impl GPUDataLog {
    pub fn new(gpu_entries: Vec<HardwareLogEntry>) -> Self {
        const TEMP_COLORS: [Color; 4] = [
            Color::from_rgb(1.0, 0.4, 0.0),   // Orange - GPU 0
            Color::from_rgb(1.0, 0.2, 0.2),   // Red - GPU 1
            Color::from_rgb(1.0, 0.0, 0.5),   // Magenta - GPU 2
            Color::from_rgb(0.9, 0.3, 0.6),   // Pink - GPU 3
        ];

        const USAGE_COLORS: [Color; 4] = [
            Color::from_rgb(0.0, 0.5, 1.0),   // Blue - GPU 0
            Color::from_rgb(0.0, 0.8, 0.8),   // Cyan - GPU 1
            Color::from_rgb(0.5, 0.0, 1.0),   // Purple - GPU 2
            Color::from_rgb(0.3, 0.6, 1.0),   // Light Blue - GPU 3
        ];

        const POWER_COLORS: [Color; 4] = [
            Color::from_rgb(1.0, 0.8, 0.0),   // Yellow - GPU 0
            Color::from_rgb(0.0, 1.0, 0.3),   // Green - GPU 1
            Color::from_rgb(0.5, 1.0, 0.0),   // Lime - GPU 2
            Color::from_rgb(1.0, 0.9, 0.2),   // Golden - GPU 3
        ];

        // Start building the plot widget
        let mut builder = PlotWidgetBuilder::new()
            .with_x_label("Time (s)")
            .with_tooltips(true)
            .with_tooltip_provider(|ctx: &TooltipContext| {
                format!("Time: {:.0}s\nValue: {:.1}", ctx.x, ctx.y)
            })
            .with_autoscale_on_updates(true)
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
            .with_cursor_provider(|x, y| format!("Time: {:.0}s\nValue: {:.1}", x, y));

        // Process GPU data if we have any entries
        if !gpu_entries.is_empty() {
            // Parse first timestamp as baseline (t=0)
            let first_ts = if let Ok(t) = DateTime::parse_from_rfc3339(&gpu_entries[0].timestamp) {
                t.timestamp()
            } else {
                0
            };

            // Group entries by model_name
            let mut gpu_groups: HashMap<String, Vec<&HardwareLogEntry>> = HashMap::new();
            for entry in gpu_entries.iter() {
                gpu_groups.entry(entry.model_name.clone()).or_default().push(entry);
            }

            // Sort GPU names for consistent ordering
            let mut gpu_names: Vec<_> = gpu_groups.keys().cloned().collect();
            gpu_names.sort();

            // Create series for each GPU
            for (gpu_idx, gpu_name) in gpu_names.iter().enumerate() {
                let entries = &gpu_groups[gpu_name];

                // Extract temperature series
                let temp_series: Vec<[f64; 2]> = entries
                    .iter()
                    .filter_map(|e| {
                        let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?;
                        let x = (ts.timestamp() - first_ts) as f64;
                        Some([x, e.temperature as f64])
                    })
                    .collect();

                // Extract usage series
                let usage_series: Vec<[f64; 2]> = entries
                    .iter()
                    .filter_map(|e| {
                        let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?;
                        let x = (ts.timestamp() - first_ts) as f64;
                        Some([x, e.usage as f64])
                    })
                    .collect();

                // Extract power series
                let power_series: Vec<[f64; 2]> = entries
                    .iter()
                    .filter_map(|e| {
                        let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?;
                        let x = (ts.timestamp() - first_ts) as f64;
                        Some([x, e.power_draw as f64])
                    })
                    .collect();

                // Add temperature series
                if !temp_series.is_empty() {
                    let temp = Series::new(
                        temp_series,
                        MarkerStyle::circle(4.0),
                        LineStyle::Solid { width: 4.0 },
                    )
                    .with_label(&format!("{} Temp (°C)", gpu_name))
                    .with_color(TEMP_COLORS[gpu_idx % TEMP_COLORS.len()]);

                    builder = builder.add_series(temp);
                }

                // Add usage series
                if !usage_series.is_empty() {
                    let usage = Series::new(
                        usage_series,
                        MarkerStyle::circle(4.0),
                        LineStyle::Solid { width: 4.0 },
                    )
                    .with_label(&format!("{} Usage (%)", gpu_name))
                    .with_color(USAGE_COLORS[gpu_idx % USAGE_COLORS.len()]);

                    builder = builder.add_series(usage);
                }

                // Add power series
                if !power_series.is_empty() {
                    let power = Series::new(
                        power_series,
                        MarkerStyle::circle(4.0),
                        LineStyle::Solid { width: 4.0 },
                    )
                    .with_label(&format!("{} Power (W)", gpu_name))
                    .with_color(POWER_COLORS[gpu_idx % POWER_COLORS.len()]);

                    builder = builder.add_series(power);
                }
            }
        } else {
            // Add dummy series if no data
            let dummy_series =
                Series::circles(vec![[0.0, 0.0]], 3.0).with_label("No GPU data available");
            builder = builder.add_series(dummy_series);
        }

        Self {
            widget: builder.build().unwrap(),
        }
    }

    pub fn view(&self) -> Element<'_, PlotUiMessage> {
        self.widget.view()
    }

    pub fn update_ui(&mut self, msg: PlotUiMessage) {
        self.widget.update(msg);
    }
}
