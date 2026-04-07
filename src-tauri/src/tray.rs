use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

/// Set up the system tray icon and menu.
/// Called once during app setup.
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let apps_item = MenuItem::with_id(app, "apps", "Manage Apps", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let toggle_item = MenuItem::with_id(
        app,
        "toggle_overlays",
        "Disable All Overlays",
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit broken-latch", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[
        &apps_item,
        &settings_item,
        &sep1,
        &toggle_item,
        &sep2,
        &quit_item,
    ])?;

    let mut builder = TrayIconBuilder::new()
        .tooltip("broken-latch Platform")
        .menu(&menu)
        .on_menu_event(|app, event: tauri::menu::MenuEvent| match event.id().as_ref() {
            "apps" => open_app_manager(app),
            "settings" => open_settings(app),
            "toggle_overlays" => toggle_all_overlays(app),
            "quit" => app.exit(0),
            _ => {}
        });

    // Use the app window icon if available
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

fn open_app_manager(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("app-manager") {
        window.show().ok();
        window.set_focus().ok();
    } else {
        tauri::WebviewWindowBuilder::new(
            app,
            "app-manager",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("broken-latch — App Manager")
        .inner_size(900.0, 650.0)
        .min_inner_size(700.0, 500.0)
        .resizable(true)
        .center()
        .build()
        .ok();
    }
}

fn open_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().ok();
        window.set_focus().ok();
    } else {
        tauri::WebviewWindowBuilder::new(
            app,
            "settings",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("broken-latch — Settings")
        .inner_size(650.0, 550.0)
        .resizable(false)
        .center()
        .build()
        .ok();
    }
}

fn toggle_all_overlays(app: &AppHandle) {
    log::info!("Toggle all overlays requested");
    // Emit event so frontend / widget manager can react
    use tauri::Emitter;
    app.emit("overlays_toggle", ()).ok();
}
