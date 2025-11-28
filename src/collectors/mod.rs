pub mod cpu_collector;
pub mod cpu_frequency_collector;
pub mod gpu_collector;
pub mod lhm_collector;
pub use gpu_collector::GpuData;

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
