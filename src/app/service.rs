use anyhow::{Context, Result};
use std::process::Command;

/// PawnIO and lhm service state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceState {
    Running,
    Stopped,
    StartPending,
    StopPending,
    Unknown,
}

// LibreHardwareMonitorService or PawnIO
pub fn get_service_state(service_name: &str) -> Result<ServiceState> {
    let output = Command::new("sc")
        .args(&["query", service_name])
        .output()
        .context("Failed to execute sc query")?;

    let status = String::from_utf8_lossy(&output.stdout);

    // Check if service exists
    if status.contains("does not exist") || status.contains("not found") {
        anyhow::bail!("Service '{}' not found", service_name);
    }

    let state = match () {
        _ if status.contains("RUNNING") => ServiceState::Running,
        _ if status.contains("STOPPED") => ServiceState::Stopped,
        _ if status.contains("START_PENDING") => ServiceState::StartPending,
        _ if status.contains("STOP_PENDING") => ServiceState::StopPending,
        _ => ServiceState::Unknown,
    };

    Ok(state)
}

impl ServiceState {
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceState::Running)
    }
}
