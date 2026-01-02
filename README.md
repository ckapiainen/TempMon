# ![project_title.png](assets/repository/header.png)

<div align="center">
<img src="assets/repository/main_page.gif" width="48%" />
<img src="assets/repository/plot_window.gif" width="48%" />
</div>

Built with [iced](https://iced.rs/), TempMon is fully native and provides lightweight real-time hardware
monitoring with a clean, minimal interface. In addition to live metrics, TempMon allows users to monitor specific
currently running processes
and log historical process and component data for later analysis.

## How It Works

TempMon uses LibreHardwareMonitor lib running as a service for most of its data:

### LibreHardwareMonitor Service

Communicates with the **[LibreHardwareMonitor (LHM)](https://github.com/jacobtread/lhm-service) service** via IPC pipe:

- 🔒 **No Admin Required** - Service runs elevated once, clients run without UAC prompts
- Provides: CPU/GPU temperatures, power consumption, voltages, fan speeds, and more

This service gets installed along with a required driver automatically using the installer.

## Features

### Current

- ✅ CPU & GPU metrics collection
- ✅ Real time plotting for CPU & GPU
- ✅ Process specific monitoring & logging
- ✅ System tray icon when minimized
- ✅ Settings for updating interval, themes, startup behavior and more
- ✅ CSV logging for historical data

### Roadmap for v1.0-v2.0

- 🚧 Historical data visualization with charts
- 🚧 Other hardware monitoring
- 🚧 Detailed system information
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

- Windows 10/11 with .NET Framework 4.7.2+
- Latest ver. PawnIO driver https://github.com/namazso/PawnIO