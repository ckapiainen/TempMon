## Automatic Installation via NSIS Installer (Recommended)

When using the NSIS installer package, PawnIO driver is installed automatically in silent mode.

For troubleshooting, check Windows Event Viewer or run `sc query PawnIO` after installation to see if driver was
installed.

## Manual Service Installation:

### User Requirements:

1. **PawnIO driver** must be installed (https://pawnio.eu/)
2. **Both lhm-service.exe and lhm-bridge.dll** must be in the same folder
3. Run ``install-service.bat`` as administrator

```
bundle/
├── lhm-service.exe 
├── lhm-bridge.dll 
├── install-service.bat
├── uninstall-service.bat
└── README.md
```

### pawnio-installer.exe

- **Purpose**: Installs PawnIO kernel driver for hardware monitoring
- **Source**: https://pawnio.com
- **Version**: 1.0.x (check official site for latest)

### lhm-service.exe

- **Purpose**: LibreHardwareMonitor Windows service
- **Source**: Built from LibreHardwareMonitorLib 0.9.5-pre528
- **Repository**: https://github.com/LibreHardwareMonitor/LibreHardwareMonitor
- **Note**: Runs as Windows service to provide hardware data

### lhm-bridge.dll

- **Purpose**: Bridge DLL for communicating with LibreHardwareMonitor service
- **Source**: Built from LibreHardwareMonitorLib 0.9.5-pre528
- **Repository**: https://github.com/LibreHardwareMonitor/LibreHardwareMonitor
- **Note**: Required runtime dependency for temp-monitor.exe