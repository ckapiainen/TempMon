use crate::collectors::GpuLHMQuery;
use lhm_client::HardwareType;

//TODO: max vec size for averages
#[derive(Debug, Clone)]
pub struct GpuData {
    first_run: bool,
    pub brand: HardwareType,
    pub name: String,
    pub core_temp: f32,
    pub core_temp_max: f32,
    pub core_temp_min: f32,
    pub core_temp_avg: Vec<f32>,
    pub memory_junction_temp: f32,
    pub memory_junction_temp_max: f32,
    pub memory_junction_temp_min: f32,
    pub memory_junction_temp_avg: Vec<f32>,
    pub core_clock: f32,
    pub memory_clock: f32,
    pub power: f32,
    pub core_load: f32,
    pub memory_used: f32,
    pub memory_total: f32,
}
impl GpuData {
    pub fn new(brand: HardwareType, name: String) -> Self {
        Self {
            first_run: true,
            brand,
            name,
            core_temp: 0.0,
            core_temp_max: 0.0,
            core_temp_min: 0.0,
            core_temp_avg: Vec::new(),
            memory_junction_temp: 0.0,
            memory_junction_temp_max: 0.0,
            memory_junction_temp_min: 0.0,
            memory_junction_temp_avg: Vec::new(),
            core_clock: 0.0,
            memory_clock: 0.0,
            power: 0.0,
            core_load: 0.0,
            memory_used: 0.0,
            memory_total: 0.0,
        }
    }
    pub fn update_lhm_data(&mut self, data: GpuLHMQuery) {
        if self.first_run {
            self.first_run = false;
            self.core_temp_max = data.core_temp;
            self.core_temp_min = data.core_temp;
            self.memory_junction_temp_max = data.memory_junction_temp;
            self.memory_junction_temp_min = data.memory_junction_temp;
        }

        self.core_temp = data.core_temp;
        self.memory_junction_temp = data.memory_junction_temp;
        self.core_clock = data.core_clock;
        self.power = data.power;
        self.core_load = data.core_load;
        self.memory_used = data.memory_used;
        self.memory_total = data.memory_total;
        self.memory_clock = data.memory_clock;
        // Track min/max values
        self.core_temp_max = self.core_temp_max.max(self.core_temp);
        self.core_temp_min = self.core_temp_min.min(self.core_temp);
        self.memory_junction_temp_max =
            self.memory_junction_temp_max.max(self.memory_junction_temp);
        self.memory_junction_temp_min =
            self.memory_junction_temp_min.min(self.memory_junction_temp);
        self.core_temp_avg.push(self.core_temp);
        self.memory_junction_temp_avg
            .push(self.memory_junction_temp);
    }
    pub fn get_core_temp_avg(&self) -> f32 {
        let avg = self.core_temp_avg.iter().sum::<f32>() / self.core_temp_avg.len() as f32;
        (avg * 100.0).round() / 100.0 // Round to 2 decimal places
    }
    pub fn get_memory_junction_temp_avg(&self) -> f32 {
        let avg = self.memory_junction_temp_avg.iter().sum::<f32>()
            / self.memory_junction_temp_avg.len() as f32;
        (avg * 100.0).round() / 100.0 // Round to 2 decimal places
    }
}
