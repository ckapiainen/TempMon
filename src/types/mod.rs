pub mod hardware;
pub mod settings;
pub mod ui;

// Re-export commonly used types
pub use hardware::{ComponentType, CpuCoreLHMQuery, GpuLHMQuery, HardwareLogEntry};
pub use settings::{Config, TempUnits};
pub use ui::CpuBarChartState;
