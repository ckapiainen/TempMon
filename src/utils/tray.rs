use tray_icon::menu::{Menu, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub fn init_icon() -> (MenuId, MenuId, TrayIcon) {
    const ICON_DATA: &[u8] = include_bytes!("../../assets/logo.ico");
    let image = image::load_from_memory(ICON_DATA)
        .expect("Failed to load icon from memory")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon = Icon::from_rgba(rgba, width, height).expect("Failed to create icon");
    // Create tray menu
    let menu = Menu::new();
    let show_item = MenuItem::new("Show Window", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let separator = PredefinedMenuItem::separator();

    // Store menu IDs for event handling
    let show_id = show_item.id().clone();
    let quit_id = quit_item.id().clone();

    menu.append_items(&[&show_item, &separator, &quit_item])
        .expect("Failed to append menu items");

    (
        show_id,
        quit_id,
        TrayIconBuilder::new()
            .with_tooltip("TempMon")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .expect("Failed to create tray icon"),
    )
}
