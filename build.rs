fn main() {
    // Embed icon resource for .exe file icon (Windows Explorer, taskbar, etc.)
    let _ = embed_resource::compile("assets/tray.rc", embed_resource::NONE);
}
