use winreg::enums::*;
use winreg::RegKey;

const APP_NAME: &str = "TempMon";

pub fn set_start_with_windows(enabled: bool) -> Result<(), std::io::Error> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")?;

    if enabled {
        let exe_path = std::env::current_exe()?;
        key.set_value(APP_NAME, &exe_path.to_string_lossy().as_ref())?;
    } else {
        let _ = key.delete_value(APP_NAME); // ignore error if key doesn't exist
    }
    Ok(())
}

pub fn is_start_with_windows_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") else {
        return false;
    };
    key.get_value::<String, _>(APP_NAME).is_ok()
}
