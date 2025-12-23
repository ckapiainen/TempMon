use serde::{Deserialize, Serialize};

/// Shared data structure for CPU core statistics (usage, power, etc.)
#[derive(Debug, Clone)]
pub struct CpuCoreLHMQuery {
    pub name: String,
    pub value: f32,
}

#[derive(Debug, Clone, Default)]
pub struct GpuLHMQuery {
    pub core_temp: f32,
    pub memory_junction_temp: f32,
    pub core_clock: f32,
    pub memory_clock: f32,
    pub power: f32,
    pub core_load: f32,
    pub memory_used: f32,
    pub memory_total: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    CPU,
    GPU,
    RAM,
    SSD,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareLogEntry {
    pub timestamp: String,
    pub selected_process: String,
    pub component_type: ComponentType,
    pub model_name: String,
    pub temperature_unit: String,
    pub temperature: f32,
    pub usage: f32,
    pub power_draw: f32,
}
