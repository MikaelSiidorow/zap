mod commands;

use log::{debug, error, info, warn};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use zap_core::PluginHost;
use zap_plugin_apps::AppsPlugin;
use zap_plugin_calc::CalcPlugin;
use zap_plugin_clipboard::ClipboardPlugin;

static LAST_SHOW_TIME: AtomicI64 = AtomicI64::new(0);

pub fn socket_path() -> std::path::PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    runtime_dir.join("zap.sock")
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

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
            gtk_window.present_with_time(0); // GDK_CURRENT_TIME = 0
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
        debug!("show_window called");
        show_main_window(&window);
    } else {
        error!("show_window: no 'main' window found");
    }
}

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
                debug!("Toggle via socket");
                toggle_window(&app);
            }
        }
    });
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "Show Zap", true, None::<&str>)?;
    let reindex_i = MenuItem::with_id(app, "reindex", "Reindex Apps", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &reindex_i, &quit_i])?;

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
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    info!("Tray icon ready");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_target(false)
        .init();

    let mut host = PluginHost::new();
    host.register(Box::new(AppsPlugin::new()));
    host.register(Box::new(CalcPlugin));
    host.register(Box::new(ClipboardPlugin::new()));
    host.init_all().expect("failed to initialize plugins");

    tauri::Builder::default()
        .setup(move |app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::ShortcutState;

                let shortcut = "alt+space";
                match tauri_plugin_global_shortcut::Builder::new()
                    .with_shortcuts([shortcut])
                {
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

            app.manage(host);

            setup_tray(app)?;

            if let Some(window) = app.get_webview_window("main") {
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
                        debug!("Blur event, {elapsed}ms since last show");
                        if elapsed > 300 {
                            hide_main_window(&w);
                        } else {
                            debug!("Blur suppressed (too soon after show)");
                        }
                    }
                });
            }

            spawn_socket_listener(app.handle().clone());

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("icon", |_ctx, request, responder| {
            std::thread::spawn(move || {
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
                    Ok(data) => {
                        responder.respond(
                            tauri::http::Response::builder()
                                .header("content-type", mime)
                                .body(data)
                                .unwrap(),
                        );
                    }
                    Err(_) => {
                        responder.respond(
                            tauri::http::Response::builder()
                                .status(404)
                                .body(Vec::new())
                                .unwrap(),
                        );
                    }
                }
            });
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::execute,
            commands::copy_to_clipboard,
            commands::hide_window,
            commands::paste_to_frontmost,
            commands::paste_image_to_frontmost,
            commands::copy_image_to_clipboard,
            commands::clipboard_delete,
            commands::clipboard_toggle_pin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
