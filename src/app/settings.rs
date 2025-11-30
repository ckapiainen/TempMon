use crate::app::modal::modal;
use crate::app::service::{get_service_state, ServiceState};
use crate::app::styles;
use crate::AppMessage;
use anyhow::{Context, Result};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule, scrollable, slider, text, text_input,
};
use iced::{Alignment, Background, Color, Element, Length, Theme};
use serde::{Deserialize, Serialize};
use std::{fmt, fs};

// Saved to disk
#[derive(Serialize, Deserialize)]
struct Config {
    theme: String,
    start_with_windows: bool,
    start_minimized: bool,
    selected_temp_units: TempUnits,
    data_update_interval: f32,
    temp_low_threshold: f32,
    temp_high_threshold: f32,
}

// Runtime settings
#[derive(Clone)]
pub struct Settings {
    pawnio_status: ServiceState,
    lhm_service_status: ServiceState,
    pub theme: Theme,
    pub start_with_windows: bool,
    pub start_minimized: bool,
    pub selected_temp_units: Option<TempUnits>,
    pub data_update_interval: f32,
    pub temp_low_threshold: f32,
    pub temp_high_threshold: f32,
    pub temp_low_input: String,
    pub temp_high_input: String,
    pub update_interval_input: String,
}
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TempUnits {
    Celsius,
    Fahrenheit,
}
impl fmt::Display for TempUnits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TempUnits::Celsius => write!(f, "Celsius"),
            TempUnits::Fahrenheit => write!(f, "Fahrenheit"),
        }
    }
}

impl TempUnits {
    pub fn convert(&self, value: f32, to_unit: TempUnits) -> f32 {
        if self == &to_unit {
            return value; // No conversion needed
        }
        match (self, to_unit) {
            (TempUnits::Celsius, TempUnits::Fahrenheit) => value * 9.0 / 5.0 + 32.0,
            (TempUnits::Fahrenheit, TempUnits::Celsius) => (value - 32.0) * 5.0 / 9.0,
            _ => value,
        }
    }

    /// Returns the symbol for this temperature unit ("°C" or "°F")
    pub fn symbol(&self) -> &'static str {
        match self {
            TempUnits::Celsius => "°C",
            TempUnits::Fahrenheit => "°F",
        }
    }

    /// Convert a Celsius value to this unit and format with symbol
    pub fn format_from_celsius(&self, celsius_value: f32, decimals: usize) -> String {
        let converted = TempUnits::Celsius.convert(celsius_value, *self);
        format!("{:.decimals$}{}", converted, self.symbol(), decimals = decimals)
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            pawnio_status: ServiceState::Unknown,
            lhm_service_status: ServiceState::Unknown,
            theme: Theme::Dracula,
            start_with_windows: true,
            start_minimized: false,
            selected_temp_units: Some(TempUnits::Celsius),
            data_update_interval: 2.0,
            temp_low_threshold: 40.0,
            temp_high_threshold: 70.0,
            temp_low_input: "40".to_string(),
            temp_high_input: "70".to_string(),
            update_interval_input: "2.0".to_string(),
        }
    }
}

// TODO: MORE settings.
// TODO: start with windows and minimized
// Tray icon:
// "Show temperature" checkbox
// "Show CPU usage" checkbox
// "Show power draw" checkbox
impl Settings {
    // Helper function to get config path in AppData
    fn get_config_path() -> std::path::PathBuf {
        if let Some(data_dir) = dirs::data_local_dir() {
            data_dir.join("TempMon").join("config").join("cfg.toml")
        } else {
            std::path::PathBuf::from("config/cfg.toml")
        }
    }

    pub fn load() -> Result<Self> {
        let pawnio = get_service_state("PawnIO").unwrap_or(ServiceState::Stopped);
        let lhm_service =
            get_service_state("LibreHardwareMonitorService").unwrap_or(ServiceState::Stopped);
        let path = Self::get_config_path();

        // Create config directory if needed
        if !path.exists() {
            let default = Self::default();
            default.save()?;
            return Ok(default);
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {:?}", path))?;
        let config: Config = toml::from_str(&contents).with_context(|| "Failed to parse config")?;

        let theme = match config.theme.as_str() {
            "Dark" => Theme::Dark,
            "Dracula" => Theme::Dracula,
            "Nord" => Theme::Nord,
            "Ferra" => Theme::Ferra,
            _ => Theme::Dracula,
        };

        dbg!("Loaded config from disk");

        // Thresholds are stored in the selected unit, use them as-is for display
        Ok(Self {
            pawnio_status: pawnio,
            lhm_service_status: lhm_service,
            theme,
            start_minimized: config.start_minimized,
            start_with_windows: config.start_with_windows,
            selected_temp_units: Some(config.selected_temp_units),
            data_update_interval: config.data_update_interval,
            temp_low_threshold: config.temp_low_threshold,
            temp_high_threshold: config.temp_high_threshold,
            temp_low_input: format!("{:.0}", config.temp_low_threshold),
            temp_high_input: format!("{:.0}", config.temp_high_threshold),
            update_interval_input: config.data_update_interval.to_string(),
        })
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_config_path();

        // Create directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let theme_name = self.theme.to_string();
        let config = Config {
            theme: theme_name,
            start_minimized: self.start_minimized,
            start_with_windows: self.start_with_windows,
            selected_temp_units: self
                .selected_temp_units
                .expect("Temp unit must be selected"),
            data_update_interval: self.data_update_interval,
            temp_low_threshold: self.temp_low_threshold,
            temp_high_threshold: self.temp_high_threshold,
        };

        let toml = toml::to_string_pretty(&config).context("Failed to serialize config")?;
        fs::write(&path, toml).with_context(|| format!("Failed to write config to {:?}", path))?;
        dbg!("Saved config to disk");
        Ok(())
    }

    /// Get the selected temperature unit, defaulting to Celsius if None
    pub fn temp_unit(&self) -> TempUnits {
        self.selected_temp_units.unwrap_or(TempUnits::Celsius)
    }

    /// Format a Celsius temperature value in the user's selected unit
    pub fn format_temp(&self, celsius_value: f32, decimals: usize) -> String {
        self.temp_unit().format_from_celsius(celsius_value, decimals)
    }

    pub fn view<'a>(&'a self, base: Element<'a, AppMessage>) -> Element<'a, AppMessage> {
        // Header with title and close button
        let header = container(
            row![
                text("Settings")
                    .size(24)
                    .width(Length::Fill)
                    .style(|_theme| text::Style {
                        color: Some(Color::from_rgb(0.9, 0.9, 0.9))
                    }),
                button(text("✕").size(20))
                    .on_press(AppMessage::HideSettingsModal)
                    .padding([4, 10])
                    .style(styles::header_button_style),
            ]
            .align_y(Alignment::Center)
            .spacing(10),
        )
        .padding([15, 20])
        .width(Length::Fill);

        /*
        ========== SERVICE STATUS SECTION ==========
        */

        let service_status_section = {
            // Helper to create status indicator
            let create_status_indicator = |label: String, state: ServiceState| {
                let (status_text, status_color) = match state {
                    ServiceState::Running => ("Running", Color::from_rgb(0.3, 0.8, 0.3)),
                    ServiceState::Stopped => ("Stopped", Color::from_rgb(0.9, 0.3, 0.3)),
                    ServiceState::StartPending => ("Starting...", Color::from_rgb(0.9, 0.7, 0.2)),
                    ServiceState::StopPending => ("Stopping...", Color::from_rgb(0.9, 0.7, 0.2)),
                    ServiceState::Unknown => ("Unknown", Color::from_rgb(0.9, 0.3, 0.3)),
                };

                column![
                    // Service name
                    text(label).size(13).style(|_theme| text::Style {
                        color: Some(Color::from_rgb(0.75, 0.75, 0.75))
                    }),
                    // Status row
                    row![
                        // Status dot
                        text("●").size(14).style(move |_theme| text::Style {
                            color: Some(status_color)
                        }),
                        // Status text
                        text(status_text).size(12).style(move |_theme| text::Style {
                            color: Some(status_color)
                        })
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                ]
                .spacing(4)
                .width(Length::Fill)
            };

            column![
                text("SERVICES").size(14).style(|_theme| text::Style {
                    color: Some(Color::from_rgb(0.6, 0.6, 0.6))
                }),
                container(
                    row![
                        create_status_indicator("PawnIO Driver".to_string(), self.pawnio_status),
                        create_status_indicator(
                            "LibreHardwareMonitor service".to_string(),
                            self.lhm_service_status
                        ),
                    ]
                    .spacing(12)
                )
                .padding(10)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.15, 0.15, 0.16, 0.5))),
                    border: iced::Border {
                        color: Color::from_rgba(0.3, 0.3, 0.35, 0.3),
                        width: 1.0,
                        radius: iced::border::Radius::from(10.0),
                    },
                    ..Default::default()
                }),
            ]
            .spacing(8)
        };

        /*
        ========== APPEARANCE SECTION ==========
        */
        let appearance_section = iced::widget::column![
            text("APPEARANCE").size(14).style(|_theme| text::Style {
                color: Some(Color::from_rgb(0.6, 0.6, 0.6))
            }),
            text("Theme").size(15).style(|_theme| text::Style {
                color: Some(Color::from_rgb(0.9, 0.9, 0.9))
            }),
            pick_list(
                [Theme::Dracula, Theme::Ferra, Theme::Dark, Theme::Nord],
                Some(&self.theme),
                AppMessage::ThemeChanged,
            )
            .width(Length::Fill)
            .padding(10),
        ]
        .spacing(8);

        // ========== BEHAVIOR SECTION ==========
        let behavior_section = iced::widget::column![
            text("BEHAVIOR").size(14).style(|_theme| text::Style {
                color: Some(Color::from_rgb(0.6, 0.6, 0.6))
            }),
            checkbox("Start with Windows", self.start_with_windows)
                .on_toggle(AppMessage::ToggleStartWithWindows),
            checkbox("Start minimized to tray", self.start_minimized)
                .on_toggle(AppMessage::ToggleStartMinimized),
            column![
                text("Update Interval")
                    .size(15)
                    .style(|_theme| text::Style {
                        color: Some(Color::from_rgb(0.9, 0.9, 0.9))
                    }),
                row![
                    slider(
                        0.5..=10.0,
                        self.data_update_interval,
                        AppMessage::UpdateIntervalChanged
                    )
                    .step(0.5)
                    .width(Length::Fill),
                    container(
                        text(format!("{:.1}s", self.data_update_interval))
                            .size(14)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.8, 0.8, 0.8))
                            })
                    )
                    .width(Length::Fixed(50.0))
                    .align_x(iced::alignment::Horizontal::Right),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text("How often to refresh hardware and line graph data.")
                    .size(12)
                    .style(|_theme| text::Style {
                        color: Some(Color::from_rgb(0.6, 0.6, 0.6))
                    }),
            ]
            .spacing(5),
        ]
        .spacing(8);

        /*
        ========== TEMPERATURE SECTION ==========
        */

        let unit = self.selected_temp_units.map(|u| match u {
            TempUnits::Celsius => "°C",
            TempUnits::Fahrenheit => "°F",
        });

        let temp_section = iced::widget::column![
            text("TEMPERATURE").size(14).style(|_theme| text::Style {
                color: Some(Color::from_rgb(0.6, 0.6, 0.6))
            }),
            column![
                text("Units").size(15).style(|_theme| text::Style {
                    color: Some(Color::from_rgb(0.9, 0.9, 0.9))
                }),
                pick_list(
                    [TempUnits::Celsius, TempUnits::Fahrenheit,],
                    self.selected_temp_units,
                    AppMessage::TempUnitSelected,
                )
                .width(140)
                .padding(10),
            ]
            .spacing(5),
            column![
                text("Thresholds").size(15).style(|_theme| text::Style {
                    color: Some(Color::from_rgb(0.9, 0.9, 0.9))
                }),
                row![
                    column![
                        text(format!("Low ({})", unit.unwrap_or("°C")))
                            .size(14)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.7, 0.7, 0.7))
                            }),
                        text_input("60", &self.temp_low_input)
                            .on_input(AppMessage::TempLowThresholdChanged)
                            .padding(10)
                            .width(Length::Fixed(80.0)),
                    ]
                    .spacing(5),
                    column![
                        text(format!("High ({})", unit.unwrap_or("°C")))
                            .size(14)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.7, 0.7, 0.7))
                            }),
                        text_input("80", &self.temp_high_input)
                            .on_input(AppMessage::TempHighThresholdChanged)
                            .padding(10)
                            .width(Length::Fixed(80.0)),
                    ]
                    .spacing(5),
                ]
                .spacing(15),
                text("Configure temperature ranges for tray icon color changes")
                    .size(12)
                    .style(|_theme| text::Style {
                        color: Some(Color::from_rgb(0.6, 0.6, 0.6))
                    }),
                text("Low: ≤ Low threshold | Medium: Between thresholds | High: ≥ High threshold")
                    .size(11)
                    .style(|_theme| text::Style {
                        color: Some(Color::from_rgb(0.55, 0.55, 0.55))
                    }),
            ]
            .spacing(5),
        ]
        .spacing(8);

        // Save button
        let save_button = button(
            text("Save Settings")
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .on_press(crate::AppMessage::SaveSettings)
        .padding(12)
        .width(Length::Fill)
        .style(styles::rounded_button_style);

        // Combine all sections
        let separator_color = Color::from_rgb(0.3, 0.3, 0.3);

        let scrollbar_config = scrollable::Scrollbar::new().scroller_width(4);
        let content = iced::widget::column![
            header,
            rule::horizontal(1),
            container(
                scrollable(
                    container(
                        column![
                            service_status_section,
                            rule::horizontal(1).style(move |_theme| rule::Style {
                                color: separator_color,
                                snap: false,
                                fill_mode: rule::FillMode::Full,
                                radius: 0.0.into(),
                            }),
                            appearance_section,
                            rule::horizontal(1).style(move |_theme| rule::Style {
                                color: separator_color,
                                snap: false,
                                fill_mode: rule::FillMode::Full,
                                radius: 0.0.into(),
                            }),
                            behavior_section,
                            rule::horizontal(1).style(move |_theme| rule::Style {
                                color: separator_color,
                                snap: false,
                                fill_mode: rule::FillMode::Full,
                                radius: 0.0.into(),
                            }),
                            temp_section,
                            rule::horizontal(1).style(move |_theme| rule::Style {
                                color: separator_color,
                                snap: false,
                                fill_mode: rule::FillMode::Full,
                                radius: 0.0.into(),
                            }),
                            save_button,
                        ]
                        .spacing(10)
                    )
                    .padding(20)
                    .width(Length::Fill),
                )
                .direction(scrollable::Direction::Vertical(scrollbar_config))
                .style(styles::thin_scrollbar_style)
            )
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        // Modal content container
        let modal_content = container(content)
            .width(550)
            .height(600)
            .style(styles::modal_generic);

        modal(base, modal_content, AppMessage::HideSettingsModal, false)
    }
}
