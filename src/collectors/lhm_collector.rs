use super::{CpuCoreLHMQuery, GpuData, GpuLHMQuery};
use lhm_client::{HardwareType, SensorType};

pub async fn lhm_cpu_queries(
    client: &lhm_client::LHMClientHandle,
) -> anyhow::Result<(f32, f32, Vec<CpuCoreLHMQuery>)> {
    // Request all CPU hardware
    let mut temp = 0.0;
    let mut total_package_power = 0.0;
    let mut core_power: Vec<CpuCoreLHMQuery> = Vec::new();

    let cpu_list = client
        .query_hardware(None, Some(HardwareType::Cpu))
        .await?;

    for cpu in cpu_list {
        // Request all CPU temperature sensors
        let total_temp_query = client
            .query_sensors(Some(cpu.identifier.clone()), Some(SensorType::Temperature))
            .await?;

        let power_query = client
            .query_sensors(Some(cpu.identifier.clone()), Some(SensorType::Power))
            .await?;

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
            });

        // If no temperature sensor found, skip this CPU
        let Some(temp_sensor) = temp_sensor else {
            eprintln!("Warning: No CPU temperature sensor found for {}", cpu.name);
            continue;
        };

        if let Some(total) = power_query
            .iter()
            .find(|sensor| sensor.name.contains("Package"))
        {
            total_package_power = total.value;
        }

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
            .await?
            .unwrap_or(0.0);
    }
    Ok((temp, total_package_power, core_power))
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
) -> anyhow::Result<GpuLHMQuery> {
    let mut gpu_data = GpuLHMQuery::default();
    let mut gpu_list = Vec::new();
    match brand {
        HardwareType::GpuNvidia => {
            gpu_list = client
                .query_hardware(None, Some(HardwareType::GpuNvidia))
                .await?;
        }
        HardwareType::GpuAmd => {
            gpu_list = client
                .query_hardware(None, Some(HardwareType::GpuAmd))
                .await?;
        }
        HardwareType::GpuIntel => {
            gpu_list = client
                .query_hardware(None, Some(HardwareType::GpuIntel))
                .await?;
        }
        _ => {
            // Unsupported GPU brand
            return Ok(gpu_data);
        }
    }

    for gpu in gpu_list {
        // Query temperature sensors
        let temp_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::Temperature))
            .await?;

        // Query clock sensors
        let clock_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::Clock))
            .await?;

        // Query power sensors
        let power_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::Power))
            .await?;

        // Query load sensors
        let load_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::Load))
            .await?;

        // Query memory (SmallData) sensors
        let memory_sensors = client
            .query_sensors(Some(gpu.identifier.clone()), Some(SensorType::SmallData))
            .await?;

        // Extract GPU Core temperature
        if let Some(sensor) = temp_sensors.iter().find(|s| s.name == "GPU Core") {
            gpu_data.core_temp = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await?
                .unwrap_or(0.0);
        }

        // Extract GPU Memory Junction temperature
        if let Some(sensor) = temp_sensors
            .iter()
            .find(|s| s.name == "GPU Memory Junction")
        {
            gpu_data.memory_junction_temp = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await?
                .unwrap_or(0.0);
        }

        // Extract GPU Core clock
        if let Some(sensor) = clock_sensors.iter().find(|s| s.name == "GPU Core") {
            gpu_data.core_clock = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await?
                .unwrap_or(0.0);
        }

        // Extract GPU Memory clock
        if let Some(sensor) = clock_sensors.iter().find(|s| s.name == "GPU Memory") {
            gpu_data.memory_clock = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await?
                .unwrap_or(0.0);
        }

        // Extract GPU Package power
        if let Some(sensor) = power_sensors.iter().find(|s| s.name == "GPU Package") {
            gpu_data.power = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await?
                .unwrap_or(0.0);
        }

        // Extract GPU Core load
        if let Some(sensor) = load_sensors.iter().find(|s| s.name == "GPU Core") {
            gpu_data.core_load = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await?
                .unwrap_or(0.0);
        }

        // Extract GPU Memory Used
        if let Some(sensor) = memory_sensors.iter().find(|s| s.name == "GPU Memory Used") {
            gpu_data.memory_used = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await?
                .unwrap_or(0.0);
        }

        // Extract GPU Memory Total
        if let Some(sensor) = memory_sensors.iter().find(|s| s.name == "GPU Memory Total") {
            gpu_data.memory_total = client
                .get_sensor_value_by_idx(sensor.index, true)
                .await?
                .unwrap_or(0.0);
        }
    }
    Ok(gpu_data)
}
