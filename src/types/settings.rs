use serde::{Deserialize, Serialize};
use std::fmt;

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
    /// Method for converting between temperature units
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
        format!(
            "{:.decimals$}{}",
            converted,
            self.symbol(),
            decimals = decimals
        )
    }
}

// Saved to disk
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub start_with_windows: bool,
    pub start_minimized: bool,
    pub selected_temp_units: TempUnits,
    pub data_update_interval: f32,
    pub temp_low_threshold: f32,
    pub temp_high_threshold: f32,
}
