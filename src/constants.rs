//! This module contains all configuration constants organized by category.

/// Animation-related constants for UI card animations
pub mod animation {
    pub const CPU_CARD_COLLAPSED_HEIGHT: f32 = 50.0;
    pub const CPU_CARD_EXPANDED_HEIGHT: f32 = 260.0;
    pub const CORES_CARD_COLLAPSED_HEIGHT: f32 = 50.0;
    pub const CORES_CARD_EXPANDED_HEIGHT: f32 = 280.0;
    pub const GPU_CARD_COLLAPSED_HEIGHT: f32 = 50.0;
    pub const GPU_CARD_EXPANDED_HEIGHT: f32 = 350.0;
}

/// Sidebar-related constants for the plot window
pub mod sidebar {
    pub const SIDEBAR_EXPANDED_WIDTH: f32 = 550.0;
    pub const SIDEBAR_COLLAPSED_WIDTH: f32 = 50.0;
}

/// Logging and data buffering constants
pub mod logging {
    /// Write buffer size in development (flush immediately)
    pub const DEV_BUFFER_SIZE: usize = 1;
    /// Write buffer size in prod
    pub const PROD_BUFFER_SIZE: usize = 50;
    /// Maximum size of graph data buffer (last N entries)
    pub const GRAPH_DATA_BUFFER_MAX: usize = 1000;
}

/// Data collection and averaging window sizes
pub mod data {
    /// Number of samples for temperature averaging
    pub const TEMP_AVG_WINDOW_SIZE: usize = 30;
    /// Number of samples for CPU usage averaging
    pub const USAGE_AVG_WINDOW_SIZE: usize = 30;
}
