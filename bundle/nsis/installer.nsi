!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"

; ===== Basic Definitions =====
!define PRODUCT_NAME "{{product_name}}"
!define PRODUCT_VERSION "{{version}}"
!define PRODUCT_PUBLISHER "{{publisher}}"
!define PRODUCT_INSTALL_DIR "$PROGRAMFILES64\${PRODUCT_NAME}"

; ===== Installer Attributes =====
Name "${PRODUCT_NAME}"
OutFile "{{out_file}}"
InstallDir "${PRODUCT_INSTALL_DIR}"
RequestExecutionLevel admin
SetCompressor /SOLID lzma

; ===== UI Configuration =====
!define MUI_ABORTWARNING
{{#if installer_icon}}
!define MUI_ICON "{{installer_icon}}"
!define MUI_UNICON "{{installer_icon}}"
{{/if}}

; ===== Pages =====
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

; ===== Installation Section =====
Section "MainSection" SEC01
    SetOutPath "$INSTDIR"

    ; Install main binary
    File "{{main_binary_path}}"

    ; Create subdirectories
    CreateDirectory "$INSTDIR\assets"
    CreateDirectory "$INSTDIR\bin"

    ; Copy all resources to root first
    {{#each resources}}
    File "{{@key}}"
    {{/each}}

    ; Move files to correct subdirectories
    Rename "$INSTDIR\logo.ico" "$INSTDIR\assets\logo.ico"
    Rename "$INSTDIR\PawnIO_setup.exe" "$INSTDIR\bin\PawnIO_setup.exe"
    Rename "$INSTDIR\lhm-service.exe" "$INSTDIR\bin\lhm-service.exe"
    Rename "$INSTDIR\lhm-bridge.dll" "$INSTDIR\bin\lhm-bridge.dll"

    SetOutPath "$INSTDIR"

    ; Check and install PawnIO driver (silent mode)
    DetailPrint "Checking for PawnIO driver..."
    ; Check if the actual driver service exists (not just registry key)
    ExecWait 'sc query PawnIO' $0
    ${If} $0 != 0
        ; Driver service not found, try alternate name
        ExecWait 'sc query PawnIO3' $0
        ${If} $0 != 0
            ; Neither service found, install PawnIO silently
            DetailPrint "PawnIO driver not found, installing silently..."
            DetailPrint "Please wait (this may take 30-60 seconds)..."

            ; Use -silent -install flags for fully automated installation
            ExecWait '"$INSTDIR\bin\PawnIO_setup.exe" -silent -install' $1

            ; Enhanced error handling
            ${If} $1 == 0
                DetailPrint "PawnIO installed successfully"
            ${ElseIf} $1 == 3010
                DetailPrint "PawnIO installed successfully (system reboot required)"
                DetailPrint "Application will function after reboot"
            ${Else}
                DetailPrint "Warning: PawnIO installation returned code $1"
                DetailPrint "The application may have limited functionality"
            ${EndIf}

            ; Verify installation
            DetailPrint "Verifying PawnIO installation..."
            Sleep 2000
            ExecWait 'sc query PawnIO' $2
            ${If} $2 == 0
                DetailPrint "PawnIO driver verified successfully"
            ${Else}
                ExecWait 'sc query PawnIO3' $2
                ${If} $2 == 0
                    DetailPrint "PawnIO3 driver verified successfully"
                ${Else}
                    DetailPrint "Note: PawnIO driver verification returned code $2"
                    DetailPrint "Driver may require system reboot to become active"
                ${EndIf}
            ${EndIf}
        ${Else}
            DetailPrint "PawnIO driver already loaded (PawnIO3)"
        ${EndIf}
    ${Else}
        DetailPrint "PawnIO driver already loaded (PawnIO)"
    ${EndIf}

    ; Check and install LibreHardwareMonitor service
    DetailPrint "Checking for LibreHardwareMonitor service..."
    ; Try to query the service (sc query returns error code if not found)
    ExecWait 'sc query LibreHardwareMonitorService' $0
    ${If} $0 != 0
        DetailPrint "LHM service not found, installing..."
        ; Create service with sc create
        ExecWait 'sc create LibreHardwareMonitorService binPath= "$INSTDIR\bin\lhm-service.exe" start= auto DisplayName= "LibreHardwareMonitor Service" type= own' $0
        ${If} $0 == 0
            DetailPrint "Service created successfully"
            ; Start the service
            ExecWait 'sc start LibreHardwareMonitorService' $0
            ${If} $0 == 0
                DetailPrint "Service started successfully"
            ${Else}
                DetailPrint "Warning: Failed to start service (error code: $0)"
            ${EndIf}
        ${Else}
            DetailPrint "Warning: Failed to create service (error code: $0)"
        ${EndIf}
    ${Else}
        DetailPrint "LHM service already installed"
    ${EndIf}

    ; Create Start Menu shortcuts
    CreateDirectory "$SMPROGRAMS\${PRODUCT_NAME}"
    CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk" \
        "$INSTDIR\temp-monitor.exe" "" "$INSTDIR\temp-monitor.exe" 0
    CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\Uninstall.lnk" \
        "$INSTDIR\uninstall.exe"

    ; Create desktop shortcut
    CreateShortcut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\temp-monitor.exe" "" "$INSTDIR\temp-monitor.exe" 0

    ; Write uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Write registry keys for Add/Remove Programs
    WriteRegStr HKEY_LOCAL_MACHINE "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" \
        "DisplayName" "${PRODUCT_NAME}"
    WriteRegStr HKEY_LOCAL_MACHINE "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" \
        "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKEY_LOCAL_MACHINE "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" \
        "DisplayIcon" "$INSTDIR\temp-monitor.exe,0"
    WriteRegStr HKEY_LOCAL_MACHINE "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" \
        "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr HKEY_LOCAL_MACHINE "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" \
        "DisplayVersion" "${PRODUCT_VERSION}"
SectionEnd

; ===== Uninstallation Section =====
Section "Uninstall"
    ; Stop and remove service (only if no other apps use it)
    MessageBox MB_YESNO "Do you want to remove the LibreHardwareMonitor service? (Choose No if other apps use it)" \
        IDNO skip_service

    ExecWait 'sc stop LibreHardwareMonitorService' $0
    ExecWait 'sc delete LibreHardwareMonitorService' $0

skip_service:
    ; Remove files and directories (recursive removal handles all subdirectories)
    Delete "$INSTDIR\temp-monitor.exe"
    Delete "$INSTDIR\uninstall.exe"
    RMDir /r "$INSTDIR"

    ; Remove shortcuts
    Delete "$SMPROGRAMS\${PRODUCT_NAME}\*.*"
    RMDir "$SMPROGRAMS\${PRODUCT_NAME}"
    Delete "$DESKTOP\${PRODUCT_NAME}.lnk"

    ; Remove registry keys
    DeleteRegKey HKEY_LOCAL_MACHINE "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
SectionEnd