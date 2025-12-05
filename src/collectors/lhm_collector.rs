use super::{CpuCoreLHMQuery, GpuData, GpuLHMQuery};
use lhm_client::{HardwareType, SensorType};

pub async fn lhm_cpu_queries(
    client: &lhm_client::LHMClientHandle,
) -> (f32, f32, Vec<CpuCoreLHMQuery>) {
    // Request all CPU hardware
    let mut temp = 0.0;
    let mut total_package_power = 0.0;
    let mut core_power: Vec<CpuCoreLHMQuery> = Vec::new();

    let cpu_list = client
        .query_hardware(None, Some(HardwareType::Cpu))
        .await
        .unwrap();

    for cpu in cpu_list {
        // Request all CPU temperature sensors
        let total_temp_query = client
            .query_sensors(Some(cpu.identifier.clone()), Some(SensorType::Temperature))
            .await
            .unwrap();

        let power_query = client
            .query_sensors(Some(cpu.identifier.clone()), Some(SensorType::Power))
            .await
            .unwrap();

        // Find the CPU temperature sensor
        // "CPU Package" (Intel), "Core (Tctl/Tdie)" (AMD), "CPU Core" (generic)
        let temp_sensor = total_temp_query
            .iter()
            .find(|sensor| {
                sensor.name.eq("CPU Package")
                    || sensor.name.eq("Core (Tctl/Tdie)")
                    || sensor.name.eq("CPU Core")
                    || sensor.name.contains("Package")
                    || sensor.name.contains("Tctl")
            })
            .expect("Missing cpu temp sensor");

        let total = power_query
            .iter()
            .find(|sensor| sensor.name.contains("Package"))
            .unwrap();
        total_package_power = total.value;

        core_power = power_query
            .iter()
            .filter(|sensor| sensor.name.contains("Core"))
            .map(|sensor| CpuCoreLHMQuery {
                name: sensor.name.clone(),
                value: sensor.value,
            })
            .collect();

        // Get the current sensor value
        temp = client
            .get_sensor_value_by_idx(temp_sensor.index, true)
            .await
            .unwrap()
            .expect("cpu temp sensor is now unavailable");
    }
    (temp, total_package_power, core_power)
}

pub async fn initialize_gpus(client: &lhm_client::LHMClientHandle) -> Vec<GpuData> {
    let mut gpus = Vec::new();

    // Query ALL hardware (None, None)
    let hardware_list = match client.query_hardware(None, None).await {
        Ok(hw) => hw,
        Err(e) => {
            eprintln!("Failed to query hardware: {}", e);
            return gpus;
        }
    };

    // Filter for GPU hardware types and create GpuData instances
    for hw in hardware_list {
        match hw.ty {
            HardwareType::GpuNvidia | HardwareType::GpuAmd | HardwareType::GpuIntel => {
                gpus.push(GpuData::new(hw.ty, hw.name.clone()));
            }
            _ => {} // Ignore non-GPU hardware
        }
    }

    gpus
}

pub async fn lhm_gpu_queries(
    brand: HardwareType,
    client: &lhm_client::LHMClientHandle,
) -> GpuLHMQuery {
    let mut gpu_data = GpuLHMQuery::default();
    let mut gpu_list = Vec::new();
    match brand {
        HardwareType::GpuNvidia => {
            gpu_list = client
                .query_hardware(None, Some(HardwareType::GpuNvidia))
                .await
                .unwrap();
        }
        HardwareType::GpuAmd => {
            gpu_list = client
                .query_hardware(None, Some(HardwareType::GpuAmd))
                .await
                .unwrap();
        }
        HardwareType::GpuIntel => {
            gpu_list = client
                .query_hardware(None, Some(HardwareType::GpuIntel))
                .await
                .unwrap();
        }
        _ => {
            // Unsupported GPU brand
            return gpu_data;
        }
    }

    for gpu in gpu_list {
        // Query temperature sensors
        let temp_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::Temperature))
            .await
            .unwrap();

        // Query clock sensors
        let clock_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::Clock))
            .await
            .unwrap();

        // Query power sensors
        let power_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::Power))
            .await
            .unwrap();

        // Query load sensors
        let load_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::Load))
            .await
            .unwrap();

        // Query memory (SmallData) sensors
        let memory_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::SmallData))
            .await
            .unwrap();

        // Extract GPU Core temperature
        if let Some(sensor) = temp_sensors.iter().find(|s| s.name == "GPU Core") {
            gpu_data.core_temp = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await
                .unwrap()
                .unwrap_or(0.0);
        }

        // Extract GPU Memory Junction temperature
        if let Some(sensor) = temp_sensors
            .iter()
            .find(|s| s.name == "GPU Memory Junction")
        {
            gpu_data.memory_junction_temp = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await
                .unwrap()
                .unwrap_or(0.0);
        }

        // Extract GPU Core clock
        if let Some(sensor) = clock_sensors.iter().find(|s| s.name == "GPU Core") {
            gpu_data.core_clock = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await
                .unwrap()
                .unwrap_or(0.0);
        }

        // Extract GPU Memory clock
        if let Some(sensor) = clock_sensors.iter().find(|s| s.name == "GPU Memory") {
            gpu_data.memory_clock = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await
                .unwrap()
                .unwrap_or(0.0);
        }

        // Extract GPU Package power
        if let Some(sensor) = power_sensors.iter().find(|s| s.name == "GPU Package") {
            gpu_data.power = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await
                .unwrap()
                .unwrap_or(0.0);
        }

        // Extract GPU Core load
        if let Some(sensor) = load_sensors.iter().find(|s| s.name == "GPU Core") {
            gpu_data.core_load = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await
                .unwrap()
                .unwrap_or(0.0);
        }

        // Extract GPU Memory Used
        if let Some(sensor) = memory_sensors.iter().find(|s| s.name == "GPU Memory Used") {
            gpu_data.memory_used = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await
                .unwrap()
                .unwrap_or(0.0);
        }

        // Extract GPU Memory Total
        if let Some(sensor) = memory_sensors.iter().find(|s| s.name == "GPU Memory Total") {
            gpu_data.memory_total = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await
                .unwrap()
                .unwrap_or(0.0);
        }
    }
    gpu_data
}
