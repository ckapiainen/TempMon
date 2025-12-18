use iced::widget::image as iced_image;
use std::collections::HashMap;
use sysinfo::Pid;

/// Caches process icons extracted from Windows executables.
pub struct IconCache {
    cache: HashMap<String, iced_image::Handle>,
    default_icon: iced_image::Handle,
}

impl IconCache {
    /// Creates a new icon cache with Windows default icon loaded.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            default_icon: Self::load_windows_default_icon(),
        }
    }

    /// Gets icon for a process, using cache or extracting by PID.
    /// Falls back to Windows default icon if extraction fails.
    pub fn get_icon(&mut self, process_name: &str, pid: Pid) -> iced_image::Handle {
        if let Some(icon) = self.cache.get(process_name) {
            return icon.clone();
        }

        let icon_handle = self.extract_icon(pid);
        self.cache.insert(process_name.to_string(), icon_handle.clone());
        icon_handle
    }

    /// Extracts icon from process by PID, returns default icon on failure.
    fn extract_icon(&self, pid: Pid) -> iced_image::Handle {
        use windows_icons::get_icon_by_process_id;

        get_icon_by_process_id(pid.as_u32())
            .ok()
            .map(Self::rgba_to_handle)
            .unwrap_or_else(|| self.default_icon.clone())
    }

    /// Loads Windows native default application icon from shell32.dll.
    fn load_windows_default_icon() -> iced_image::Handle {
        use windows_icons::{DllIcon, get_icon_by_dll};

        let dll_icon = DllIcon::new().with_shell32(3);

        get_icon_by_dll(dll_icon)
            .ok()
            .map(Self::rgba_to_handle)
            .unwrap_or_else(Self::create_gray_fallback)
    }

    /// Converts RgbaImage to iced image handle.
    fn rgba_to_handle(rgba_img: image::RgbaImage) -> iced_image::Handle {
        iced_image::Handle::from_rgba(
            rgba_img.width(),
            rgba_img.height(),
            rgba_img.into_raw(),
        )
    }

    /// Creates a 16x16 gray square as ultimate fallback icon.
    fn create_gray_fallback() -> iced_image::Handle {
        let pixels: Vec<u8> = [150, 150, 150, 255]
            .iter()
            .cycle()
            .take(16 * 16 * 4)
            .copied()
            .collect();

        iced_image::Handle::from_rgba(16, 16, pixels)
    }

    /// Gets the default Windows icon directly (cached, already loaded).
    pub fn get_default_icon(&self) -> iced_image::Handle {
        self.default_icon.clone()
    }
}
