mod commands;
mod config;

use log::{debug, error, info, warn};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri_plugin_autostart::ManagerExt;
use zap_core::PluginHost;
use zap_plugin_apps::AppsPlugin;
use zap_plugin_calc::CalcPlugin;
use zap_plugin_clipboard::ClipboardPlugin;
use zap_plugin_commands::CommandsPlugin;
use zap_plugin_emoji::EmojiPlugin;
use zap_plugin_websearch::WebSearchPlugin;
use zap_plugin_windows::WindowsPlugin;

static LAST_SHOW_TIME: AtomicI64 = AtomicI64::new(0);

#[cfg(unix)]
pub fn socket_path() -> std::path::PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    runtime_dir.join("zap.sock")
}

#[cfg(windows)]
pub const IPC_PORT: u16 = 52583;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Window management
// ---------------------------------------------------------------------------

pub fn hide_main_window(window: &tauri::WebviewWindow) {
    debug!("hiding window");
    let _ = window.hide();
}

fn show_main_window(window: &tauri::WebviewWindow) {
    debug!("showing window");
    LAST_SHOW_TIME.store(now_ms(), Ordering::Relaxed);
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();

    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::GtkWindowExt;
        if let Ok(gtk_window) = window.gtk_window() {
            gtk_window.set_urgency_hint(true);
            gtk_window.set_keep_above(true);
            gtk_window.present_with_time(0);
        }
    }

    let _ = window.emit("show-window", ());
}

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        debug!("toggle_window: visible={visible}");
        if visible {
            hide_main_window(&window);
        } else {
            show_main_window(&window);
        }
    } else {
        error!("toggle_window: no 'main' window found");
    }
}

fn show_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        show_main_window(&window);
    }
}

// ---------------------------------------------------------------------------
// IPC listener
// ---------------------------------------------------------------------------

#[cfg(unix)]
fn spawn_socket_listener(app: tauri::AppHandle) {
    use std::os::unix::net::UnixListener;

    let path = socket_path();
    let _ = std::fs::remove_file(&path);

    std::thread::spawn(move || {
        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind socket: {e}");
                return;
            }
        };
        info!("Socket listener ready at {}", path.display());
        for stream in listener.incoming() {
            if stream.is_ok() {
                toggle_window(&app);
            }
        }
    });
}

#[cfg(windows)]
fn spawn_socket_listener(app: tauri::AppHandle) {
    use std::net::TcpListener;

    std::thread::spawn(move || {
        let listener = match TcpListener::bind(("127.0.0.1", IPC_PORT)) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind TCP listener: {e}");
                return;
            }
        };
        info!("IPC listener ready on 127.0.0.1:{IPC_PORT}");
        for stream in listener.incoming() {
            if stream.is_ok() {
                toggle_window(&app);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Setup helpers
// ---------------------------------------------------------------------------

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "Show Zap", true, None::<&str>)?;
    let reindex_i = MenuItem::with_id(app, "reindex", "Reindex Apps", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart_i =
        CheckMenuItem::with_id(app, "autostart", "Launch at Login", true, autostart_enabled, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show_i,
            &reindex_i,
            &separator,
            &autostart_i,
            &separator,
            &quit_i,
        ],
    )?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_window(app),
            "reindex" => {
                info!("Reindexing apps...");
                let host = app.state::<PluginHost>();
                host.refresh_all();
                info!("Reindex complete");
            }
            "autostart" => {
                let manager = app.autolaunch();
                let currently_enabled = manager.is_enabled().unwrap_or(false);
                if currently_enabled {
                    if let Err(e) = manager.disable() {
                        error!("Failed to disable autostart: {e}");
                    } else {
                        info!("Autostart disabled");
                    }
                } else if let Err(e) = manager.enable() {
                    error!("Failed to enable autostart: {e}");
                } else {
                    info!("Autostart enabled");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    info!("Tray icon ready");
    Ok(())
}

#[cfg(desktop)]
fn setup_shortcut(app: &tauri::App) {
    use tauri_plugin_global_shortcut::ShortcutState;

    #[cfg(target_os = "windows")]
    let shortcut = "ctrl+space";
    #[cfg(not(target_os = "windows"))]
    let shortcut = "alt+space";

    match tauri_plugin_global_shortcut::Builder::new().with_shortcuts([shortcut]) {
        Ok(builder) => {
            if let Err(e) = app.handle().plugin(
                builder
                    .with_handler(|app, _shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            toggle_window(app);
                        }
                    })
                    .build(),
            ) {
                warn!("Failed to register shortcut '{shortcut}': {e}");
                warn!("Use `zap --toggle` or the tray icon instead");
            } else {
                info!("Global shortcut '{shortcut}' registered");
            }
        }
        Err(e) => {
            warn!("Failed to parse shortcut '{shortcut}': {e}");
            warn!("Use `zap --toggle` or the tray icon instead");
        }
    }
}

fn setup_window(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::GtkWindowExt;
        if let Ok(gtk_window) = window.gtk_window() {
            gtk_window.set_skip_pager_hint(true);
            gtk_window.set_skip_taskbar_hint(true);
        }
    }

    let w = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let elapsed = now_ms() - LAST_SHOW_TIME.load(Ordering::Relaxed);
            if elapsed > 300 {
                hide_main_window(&w);
            }
        }
    });
}

fn serve_icon(request: tauri::http::Request<Vec<u8>>, responder: tauri::UriSchemeResponder) {
    let path = percent_encoding::percent_decode(
        request.uri().path().as_bytes().get(1..).unwrap_or_default(),
    )
    .decode_utf8_lossy()
    .to_string();

    let mime = match std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
    {
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("xpm") => {
            responder.respond(
                tauri::http::Response::builder()
                    .status(404)
                    .body(Vec::new())
                    .unwrap(),
            );
            return;
        }
        _ => "image/png",
    };

    match std::fs::read(&path) {
        Ok(data) => responder.respond(
            tauri::http::Response::builder()
                .header("content-type", mime)
                .body(data)
                .unwrap(),
        ),
        Err(_) => responder.respond(
            tauri::http::Response::builder()
                .status(404)
                .body(Vec::new())
                .unwrap(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_target(false)
        .write_style(env_logger::WriteStyle::Always)
        .init();

    let plugin_config = config::load_config();
    info!("Loaded config with {} sections", plugin_config.len());

    let mut host = PluginHost::new();
    host.register(Box::new(AppsPlugin::new()));
    host.register(Box::new(CalcPlugin));
    host.register(Box::new(ClipboardPlugin::new()));
    host.register(Box::new(CommandsPlugin::new()));
    host.register(Box::new(EmojiPlugin::new()));
    host.register(Box::new(WebSearchPlugin::new()));
    host.register(Box::new(WindowsPlugin::new()));
    host.init_all(&plugin_config)
        .expect("failed to initialize plugins");

    let specta_builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
            commands::search,
            commands::plugin_hints,
            commands::execute,
            commands::delete_result,
            commands::toggle_pin,
            commands::open_url,
            commands::copy_to_clipboard,
            commands::hide_window,
            commands::paste_to_frontmost,
            commands::paste_image_to_frontmost,
            commands::copy_image_to_clipboard,
            commands::get_autostart,
            commands::set_autostart,
        ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(
            &specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);

            // Enable autostart by default on first launch
            let autolaunch = app.autolaunch();
            if !autolaunch.is_enabled().unwrap_or(true) {
                if let Err(e) = autolaunch.enable() {
                    warn!("Failed to enable autostart: {e}");
                } else {
                    info!("Autostart enabled (first launch)");
                }
            }

            #[cfg(desktop)]
            setup_shortcut(app);

            app.manage(host);
            setup_tray(app)?;

            if let Some(window) = app.get_webview_window("main") {
                setup_window(&window);
            }

            spawn_socket_listener(app.handle().clone());
            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("icon", |_ctx, request, responder| {
            std::thread::spawn(move || serve_icon(request, responder));
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
