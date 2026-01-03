use crate::types::HardwareLogEntry;
use chrono::DateTime;
use iced::{Color, Element};
use iced_plot::{
    LineStyle, MarkerStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series, Tick, TickWeight,
    TooltipContext,
};
use std::collections::HashMap;

const GAP_THRESHOLD_MINUTES: f64 = 1.0;

pub struct CPUDataLog {
    widget: PlotWidget,
}

pub struct GPUDataLog {
    widget: PlotWidget,
}

/// Split a series of points into segments, breaking when gaps exceed the threshold.
/// Returns a vector of segments, where each segment is a continuous run of data.
fn split_into_segments(points: Vec<[f64; 2]>) -> Vec<Vec<[f64; 2]>> {
    if points.is_empty() {
        return vec![];
    }

    let mut segments = Vec::new();
    let mut current_segment = vec![points[0]];

    for i in 1..points.len() {
        let gap = points[i][0] - points[i - 1][0];
        if gap > GAP_THRESHOLD_MINUTES {
            // Gap detected, start a new segment
            if !current_segment.is_empty() {
                segments.push(current_segment);
            }
            current_segment = vec![points[i]];
        } else {
            current_segment.push(points[i]);
        }
    }

    if !current_segment.is_empty() {
        segments.push(current_segment);
    }

    segments
}

impl CPUDataLog {
    pub fn new(cpu_entries: Vec<HardwareLogEntry>) -> Self {
        // Single CPU colors - using different shades
        const TEMP_COLOR: Color = Color::from_rgb(1.0, 0.3, 0.0); // Red-Orange
        const USAGE_COLOR: Color = Color::from_rgb(0.0, 0.7, 1.0); // Sky Blue
        const POWER_COLOR: Color = Color::from_rgb(1.0, 0.7, 0.0); // Orange

        // For cursor tooltip: track temperature unit changes and capture first timestamp
        let first_ts = if !cpu_entries.is_empty() {
            if let Ok(t) = DateTime::parse_from_rfc3339(&cpu_entries[0].timestamp) {
                t.timestamp()
            } else {
                0
            }
        } else {
            0
        };

        let mut unit_changes: Vec<(f64, String)> = Vec::new();
        if !cpu_entries.is_empty() {
            let mut last_unit = String::new();
            for entry in &cpu_entries {
                if entry.temperature_unit != last_unit {
                    if let Ok(ts) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                        let time_min = (ts.timestamp() - first_ts) as f64 / 60.0;
                        unit_changes.push((time_min, entry.temperature_unit.clone()));
                        last_unit = entry.temperature_unit.clone();
                    }
                }
            }
        }

        // Clone for use in both closures
        let unit_changes_cursor = unit_changes.clone();
        let first_ts_cursor = first_ts;

        // Find the temperature unit for a given time (uses the most recent unit change)
        let find_unit = |time: f64, changes: &[(f64, String)]| -> String {
            changes
                .iter()
                .rev()
                .find(|(t, _)| *t <= time)
                .map(|(_, unit)| unit.clone())
                .unwrap_or_else(|| "C".to_string())
        };

        // Format actual time from relative minutes
        let format_time = |minutes: f64, base_ts: i64| -> String {
            let actual_ts = base_ts + (minutes * 60.0) as i64;
            let dt = chrono::DateTime::from_timestamp(actual_ts, 0)
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
            dt.format("%H:%M:%S").to_string()
        };

        // Start building the plot widget
        let mut builder = PlotWidgetBuilder::new()
            .with_x_label("Time (min)")
            .with_tooltips(true)
            .with_tooltip_provider(move |ctx: &TooltipContext| {
                let unit = find_unit(ctx.x, &unit_changes);
                let time_str = format_time(ctx.x, first_ts);
                format!(
                    "{} ({:.1} min)\nValue: {:.1} °{}",
                    time_str, ctx.x, ctx.y, unit
                )
            })
            .with_autoscale_on_updates(true)
            .with_y_tick_producer(|min, max| {
                let tick_interval = 10.0;
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
            .with_y_tick_formatter(|tick| format!("{:.0}", tick.value))
            .with_tick_label_size(10.0)
            .with_crosshairs(true)
            .with_cursor_provider(move |x, y| {
                let unit = find_unit(x, &unit_changes_cursor);
                let time_str = format_time(x, first_ts_cursor);
                format!("{} ({:.1} min)\nValue: {:.1} °{}", time_str, x, y, unit)
            });

        // Process CPU data if we have any entries
        if !cpu_entries.is_empty() {
            // Parse first timestamp as baseline (t=0)
            let first_ts = if let Ok(t) = DateTime::parse_from_rfc3339(&cpu_entries[0].timestamp) {
                t.timestamp()
            } else {
                0
            };

            // Get CPU name from first entry
            let cpu_name = &cpu_entries[0].model_name;

            // Extract temperature series (x in minutes)
            let temp_series: Vec<[f64; 2]> = cpu_entries
                .iter()
                .filter_map(|e| {
                    let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?;
                    let x = (ts.timestamp() - first_ts) as f64 / 60.0;
                    Some([x, e.temperature as f64])
                })
                .collect();

            // Extract usage series (x in minutes)
            let usage_series: Vec<[f64; 2]> = cpu_entries
                .iter()
                .filter_map(|e| {
                    let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?;
                    let x = (ts.timestamp() - first_ts) as f64 / 60.0;
                    Some([x, e.usage as f64])
                })
                .collect();

            // Extract power series (x in minutes)
            let power_series: Vec<[f64; 2]> = cpu_entries
                .iter()
                .filter_map(|e| {
                    let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?;
                    let x = (ts.timestamp() - first_ts) as f64 / 60.0;
                    Some([x, e.power_draw as f64])
                })
                .collect();

            // Add temperature series (split into segments to avoid lines across gaps)
            let temp_segments = split_into_segments(temp_series);
            for (seg_idx, segment) in temp_segments.into_iter().enumerate() {
                let mut series = Series::new(
                    segment,
                    MarkerStyle::circle(1.0),
                    LineStyle::Solid { width: 1.5 },
                )
                .with_color(TEMP_COLOR);

                // Only label the first segment
                if seg_idx == 0 {
                    series = series.with_label(&format!("{} Temp", cpu_name));
                }
                builder = builder.add_series(series);
            }

            // Add usage series (split into segments)
            let usage_segments = split_into_segments(usage_series);
            for (seg_idx, segment) in usage_segments.into_iter().enumerate() {
                let mut series = Series::new(
                    segment,
                    MarkerStyle::circle(1.0),
                    LineStyle::Solid { width: 1.5 },
                )
                .with_color(USAGE_COLOR);

                if seg_idx == 0 {
                    series = series.with_label(&format!("{} Usage (%)", cpu_name));
                }
                builder = builder.add_series(series);
            }

            // Add power series (split into segments)
            let power_segments = split_into_segments(power_series);
            for (seg_idx, segment) in power_segments.into_iter().enumerate() {
                let mut series = Series::new(
                    segment,
                    MarkerStyle::circle(1.0),
                    LineStyle::Solid { width: 1.5 },
                )
                .with_color(POWER_COLOR);

                if seg_idx == 0 {
                    series = series.with_label(&format!("{} Power (W)", cpu_name));
                }
                builder = builder.add_series(series);
            }
        } else {
            // Add dummy series if no data
            let dummy_series =
                Series::circles(vec![[0.0, 0.0]], 3.0).with_label("No CPU data available");
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

impl GPUDataLog {
    pub fn new(gpu_entries: Vec<HardwareLogEntry>) -> Self {
        const TEMP_COLORS: [Color; 4] = [
            Color::from_rgb(1.0, 0.4, 0.0), // Orange - GPU 0
            Color::from_rgb(1.0, 0.2, 0.2), // Red - GPU 1
            Color::from_rgb(1.0, 0.0, 0.5), // Magenta - GPU 2
            Color::from_rgb(0.9, 0.3, 0.6), // Pink - GPU 3
        ];

        const USAGE_COLORS: [Color; 4] = [
            Color::from_rgb(0.0, 0.5, 1.0), // Blue - GPU 0
            Color::from_rgb(0.0, 0.8, 0.8), // Cyan - GPU 1
            Color::from_rgb(0.5, 0.0, 1.0), // Purple - GPU 2
            Color::from_rgb(0.3, 0.6, 1.0), // Light Blue - GPU 3
        ];

        const POWER_COLORS: [Color; 4] = [
            Color::from_rgb(1.0, 0.8, 0.0), // Yellow - GPU 0
            Color::from_rgb(0.0, 1.0, 0.3), // Green - GPU 1
            Color::from_rgb(0.5, 1.0, 0.0), // Lime - GPU 2
            Color::from_rgb(1.0, 0.9, 0.2), // Golden - GPU 3
        ];

        // For cursor tooltip: track temperature unit changes and capture first timestamp
        let first_ts = if !gpu_entries.is_empty() {
            if let Ok(t) = DateTime::parse_from_rfc3339(&gpu_entries[0].timestamp) {
                t.timestamp()
            } else {
                0
            }
        } else {
            0
        };

        let mut unit_changes: Vec<(f64, String)> = Vec::new();
        if !gpu_entries.is_empty() {
            let mut last_unit = String::new();
            for entry in &gpu_entries {
                if entry.temperature_unit != last_unit {
                    if let Ok(ts) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                        let time_min = (ts.timestamp() - first_ts) as f64 / 60.0;
                        unit_changes.push((time_min, entry.temperature_unit.clone()));
                        last_unit = entry.temperature_unit.clone();
                    }
                }
            }
        }

        // Clone for use in both closures
        let unit_changes_cursor = unit_changes.clone();
        let first_ts_cursor = first_ts;

        // Find the temperature unit for a given time (uses the most recent unit change)
        let find_unit = |time: f64, changes: &[(f64, String)]| -> String {
            changes
                .iter()
                .rev()
                .find(|(t, _)| *t <= time)
                .map(|(_, unit)| unit.clone())
                .unwrap_or_else(|| "C".to_string())
        };

        // Format actual time from relative minutes
        let format_time = |minutes: f64, base_ts: i64| -> String {
            let actual_ts = base_ts + (minutes * 60.0) as i64;
            let dt = chrono::DateTime::from_timestamp(actual_ts, 0)
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
            dt.format("%H:%M:%S").to_string()
        };

        // Start building the plot widget
        let mut builder = PlotWidgetBuilder::new()
            .with_x_label("Time (min)")
            .with_tooltips(true)
            .with_tooltip_provider(move |ctx: &TooltipContext| {
                let unit = find_unit(ctx.x, &unit_changes);
                let time_str = format_time(ctx.x, first_ts);
                format!(
                    "{} ({:.1} min)\nValue: {:.1} °{}",
                    time_str, ctx.x, ctx.y, unit
                )
            })
            .with_autoscale_on_updates(true)
            .with_y_tick_producer(|min, max| {
                let tick_interval = 10.0;
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
            .with_y_tick_formatter(|tick| format!("{:.0}", tick.value))
            .with_tick_label_size(10.0)
            .with_crosshairs(true)
            .with_cursor_provider(move |x, y| {
                let unit = find_unit(x, &unit_changes_cursor);
                let time_str = format_time(x, first_ts_cursor);
                format!("{} ({:.1} min)\nValue: {:.1} °{}", time_str, x, y, unit)
            });

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
                gpu_groups
                    .entry(entry.model_name.clone())
                    .or_default()
                    .push(entry);
            }

            // Sort GPU names for consistent ordering
            let mut gpu_names: Vec<_> = gpu_groups.keys().cloned().collect();
            gpu_names.sort();

            // Create series for each GPU
            for (gpu_idx, gpu_name) in gpu_names.iter().enumerate() {
                let entries = &gpu_groups[gpu_name];

                // Extract temperature series (x in minutes)
                let temp_series: Vec<[f64; 2]> = entries
                    .iter()
                    .filter_map(|e| {
                        let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?;
                        let x = (ts.timestamp() - first_ts) as f64 / 60.0;
                        Some([x, e.temperature as f64])
                    })
                    .collect();

                // Extract usage series (x in minutes)
                let usage_series: Vec<[f64; 2]> = entries
                    .iter()
                    .filter_map(|e| {
                        let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?;
                        let x = (ts.timestamp() - first_ts) as f64 / 60.0;
                        Some([x, e.usage as f64])
                    })
                    .collect();

                // Extract power series (x in minutes)
                let power_series: Vec<[f64; 2]> = entries
                    .iter()
                    .filter_map(|e| {
                        let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?;
                        let x = (ts.timestamp() - first_ts) as f64 / 60.0;
                        Some([x, e.power_draw as f64])
                    })
                    .collect();
                // Add temperature series (split into segments to avoid lines across gaps)
                let temp_segments = split_into_segments(temp_series);
                for (seg_idx, segment) in temp_segments.into_iter().enumerate() {
                    let mut series = Series::new(
                        segment,
                        MarkerStyle::circle(1.0),
                        LineStyle::Solid { width: 1.5 },
                    )
                    .with_color(TEMP_COLORS[gpu_idx % TEMP_COLORS.len()]);

                    // Only label the first segment
                    if seg_idx == 0 {
                        series = series.with_label(&format!("{} Temp (°C)", gpu_name));
                    }
                    builder = builder.add_series(series);
                }

                // Add usage series (split into segments)
                let usage_segments = split_into_segments(usage_series);
                for (seg_idx, segment) in usage_segments.into_iter().enumerate() {
                    let mut series = Series::new(
                        segment,
                        MarkerStyle::circle(1.0),
                        LineStyle::Solid { width: 1.5 },
                    )
                    .with_color(USAGE_COLORS[gpu_idx % USAGE_COLORS.len()]);

                    if seg_idx == 0 {
                        series = series.with_label(&format!("{} Usage (%)", gpu_name));
                    }
                    builder = builder.add_series(series);
                }

                // Add power series (split into segments)
                let power_segments = split_into_segments(power_series);
                for (seg_idx, segment) in power_segments.into_iter().enumerate() {
                    let mut series = Series::new(
                        segment,
                        MarkerStyle::circle(1.0),
                        LineStyle::Solid { width: 1.5 },
                    )
                    .with_color(POWER_COLORS[gpu_idx % POWER_COLORS.len()]);

                    if seg_idx == 0 {
                        series = series.with_label(&format!("{} Power (W)", gpu_name));
                    }
                    builder = builder.add_series(series);
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
