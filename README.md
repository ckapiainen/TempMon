# ![project_title.png](assets/repository/project_title.png)

Built with Rust and the [iced](https://iced.rs/) GUI framework, TempMon provides lightweight real-time hardware
monitoring with a clean, minimal interface.

## How It Works

TempMon uses multiple data sources for comprehensive hardware monitoring:

### LibreHardwareMonitor Service

Communicates with the **[LibreHardwareMonitor (LHM)](https://github.com/jacobtread/lhm-service) service** via IPC pipe:

- 🔒 **No Admin Required** - Service runs elevated once, clients run without UAC prompts
- Provides: CPU/GPU temperatures, power consumption, voltages, fan speeds, and more

### Win32 Performance API

Direct Win32 API PDH queries for real-time metrics:

- 📊 **CPU Frequency** - Real-time processor performance monitoring
- Planned to work independently as fallback with ```sysinfo``` if LHM service is unavailable

### Sysinfo

Cross-platform system information library for basic CPU metrics:

- 💻 **CPU Information** - Name, core count, base frequency
- 📈 **Usage Monitoring** - Global and per-core CPU utilization
- Lightweight baseline metrics collection

## Features

### Current

- ✅ CPU metrics collection (temperature, usage, power draw, frequency)
- ✅ System tray icon when minimized
- ✅ CSV logging for historical data
- ✅ Real time plotting for CPU

### Planned for v1.0-v2.0

- 🚧 Real-time and historical data visualization with charts
- 🚧 GPU and other hardware monitoring
- 🚧 Detailed system information
- 🚧 Process specific logging
- 🚧 Application state persistence
- 🚧 Linux port
- And more...

## Download

Download the latest release from the [releases page](https://github.com/ckapiainen/temp-monitor/releases).

## Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run
cargo run --release
```

## Requirements

- Windows 10/11
- Latest ver. PawnIO driver https://github.com/namazso/PawnIO
- Rust 1.70+