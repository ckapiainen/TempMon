use super::CpuCoreLHMQuery;
use crate::collectors::cpu_frequency_collector::FrequencyMonitor;
use sysinfo::System;

//TODO: max vec size for averages
pub struct CpuData {
    first_run: bool,
    pub name: String,
    pub core_count: u32,
    pub base_cpu_frequency: f64,
    pub temp: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub temp_avg: Vec<f32>,
    pub usage: f32,
    pub usage_min: f32,
    pub usage_max: f32,
    pub usage_avg: Vec<f32>,
    pub core_utilization: Vec<CpuCoreLHMQuery>,
    pub total_power_draw: f32,
    pub core_power_draw: Vec<CpuCoreLHMQuery>,
    frequency_monitor: Option<FrequencyMonitor>,
    pub current_frequency: f64,
}

impl CpuData {
    pub fn new(sys: &System) -> Self {
        let base_freq = sys.cpus()[0].frequency() as f64 / 1000.0;
        let frequency_monitor = FrequencyMonitor::new(base_freq).ok(); // If it fails just use base frequency

        let cores: Vec<CpuCoreLHMQuery> = sys
            .cpus()
            .iter()
            .map(|cpu| CpuCoreLHMQuery {
                name: cpu.name().to_string(),
                value: cpu.cpu_usage(),
            })
            .collect();

        Self {
            first_run: true,
            name: sys.cpus()[0]
                .brand()
                .trim()
                .replace("Processor", "")
                .to_string(),
            core_count: sys.cpus().len() as u32,
            base_cpu_frequency: base_freq,
            temp: 0.0,
            temp_min: 0.0,
            temp_max: 0.0,
            total_power_draw: 0.0,
            core_power_draw: Vec::new(),
            usage: sys.global_cpu_usage(),
            usage_min: sys.global_cpu_usage(),
            usage_max: sys.global_cpu_usage(),
            usage_avg: Vec::new(),
            core_utilization: cores,
            frequency_monitor,
            current_frequency: base_freq,
            temp_avg: Vec::new(),
        }
    }

    // lhm service updates
    pub fn update_lhm_data(&mut self, temps: (f32, f32, Vec<CpuCoreLHMQuery>)) {
        if self.first_run {
            self.first_run = false;
            self.temp_min = temps.0;
        }
        self.temp = temps.0;
        self.total_power_draw = temps.1;
        self.core_power_draw = temps.2;
        self.temp_max = self.temp_max.max(self.temp);
        self.temp_min = self.temp_min.min(self.temp);
        self.temp_avg.push(self.temp);
        if self.temp_avg.len() > 30 {
            self.temp_avg.remove(0);
        }
    }

    // Method to update sysinfo and win32 api data
    pub fn update(&mut self, sys: &mut System) {
        sys.refresh_cpu_all();
        let usage_update = sys.global_cpu_usage();
        self.usage = usage_update;
        self.usage_avg.push(usage_update);
        self.usage_max = self.usage_max.max(usage_update);
        self.usage_min = self.usage_min.min(usage_update);
        if self.usage_avg.len() > 30 {
            self.usage_avg.remove(0);
        }

        for (i, cpu) in sys.cpus().iter().enumerate() {
            if let Some(core_data) = self.core_utilization.get_mut(i) {
                core_data.value = cpu.cpu_usage();
            }
        }
        if let Some(ref monitor) = self.frequency_monitor {
            if let Ok(freq) = monitor.get_current_frequency() {
                self.current_frequency = freq;
            }
        }
    }

    pub fn get_temp_avg(&self) -> f32 {
        if self.temp_avg.is_empty() {
            return self.temp;
        }
        let avg = self.temp_avg.iter().sum::<f32>() / self.temp_avg.len() as f32;
        (avg * 10.0).round() / 10.0 // Round to 1 decimal place
    }

    pub fn get_usage_avg(&self) -> f32 {
        if self.usage_avg.is_empty() {
            return self.usage;
        }
        let avg = self.usage_avg.iter().sum::<f32>() / self.usage_avg.len() as f32;
        (avg * 100.0).round() / 100.0 // Round to 2 decimal places
    }
}
