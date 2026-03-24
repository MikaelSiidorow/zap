use super::{WindowEntry, WindowPlatform};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, Atom, ConnectionExt, Window};
use zap_core::icons::{desktop_info_for_class, resolve_icon};

pub struct LinuxWindowPlatform;

impl WindowPlatform for LinuxWindowPlatform {
    fn list_windows(&self) -> Vec<WindowEntry> {
        match list_x11_windows() {
            Ok(windows) => windows,
            Err(e) => {
                log::warn!("Failed to list X11 windows: {e}");
                vec![]
            }
        }
    }

    fn activate_window(&self, window_id: u64) -> anyhow::Result<()> {
        activate_x11_window(window_id as Window)
    }
}

fn list_x11_windows() -> anyhow::Result<Vec<WindowEntry>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;

    let net_client_list = intern_atom(&conn, b"_NET_CLIENT_LIST")?;
    let net_wm_name = intern_atom(&conn, b"_NET_WM_NAME")?;
    let utf8_string = intern_atom(&conn, b"UTF8_STRING")?;
    let gtk_app_id = intern_atom(&conn, b"_GTK_APPLICATION_ID")?;

    // Get window list from _NET_CLIENT_LIST (use AnyPropertyType for compatibility)
    let reply = conn
        .get_property(false, root, net_client_list, 0u32, 0, u32::MAX)?
        .reply()?;

    let window_ids: Vec<Window> = reply
        .value32()
        .map(|iter| iter.collect())
        .unwrap_or_default();

    log::debug!("_NET_CLIENT_LIST: {} windows", window_ids.len());

    let mut entries = Vec::with_capacity(window_ids.len());
    for wid in window_ids {
        let title = get_window_title(&conn, wid, net_wm_name, utf8_string).unwrap_or_default();
        // Try _GTK_APPLICATION_ID first (set by GTK apps, maps directly to .desktop file)
        let gtk_id = get_string_prop(&conn, wid, gtk_app_id, utf8_string);

        let (app_name, icon_path) = {
            let wm = get_wm_class_parts(&conn, wid);
            let (instance, class) = wm
                .as_ref()
                .map(|(i, c)| (i.as_str(), c.as_str()))
                .unwrap_or_default();

            // Try desktop lookup: _GTK_APPLICATION_ID first, then StartupWMClass
            let info = gtk_id
                .as_deref()
                .and_then(|id| desktop_info_for_class(id, id))
                .or_else(|| desktop_info_for_class(instance, class));

            if let Some(info) = info {
                (info.name, info.icon_path)
            } else {
                let icon = resolve_icon(instance)
                    .or_else(|| resolve_icon(class))
                    .or_else(|| resolve_icon(&instance.to_lowercase()));
                (friendly_name(class), icon)
            }
        };
        log::debug!(
            "  wid=0x{:x} title='{}' app='{}'",
            wid,
            if title.is_empty() { "(empty)" } else { &title },
            if app_name.is_empty() {
                "(empty)"
            } else {
                &app_name
            },
        );

        if title.is_empty() {
            continue;
        }

        entries.push(WindowEntry {
            window_id: wid as u64,
            title,
            app_name,
            icon_path,
        });
    }

    Ok(entries)
}

fn activate_x11_window(window_id: Window) -> anyhow::Result<()> {
    // Use xdotool which handles GNOME/mutter quirks (XSetInputFocus + XRaiseWindow)
    let status = std::process::Command::new("xdotool")
        .arg("windowactivate")
        .arg(window_id.to_string())
        .status()?;
    if !status.success() {
        anyhow::bail!("xdotool windowactivate failed for window {window_id}");
    }
    Ok(())
}

fn get_string_prop(
    conn: &impl Connection,
    window: Window,
    prop: Atom,
    type_: Atom,
) -> Option<String> {
    let reply = conn
        .get_property(false, window, prop, type_, 0, 1024)
        .ok()?
        .reply()
        .ok()?;
    if reply.value.is_empty() {
        return None;
    }
    Some(String::from_utf8_lossy(&reply.value).to_string())
}

fn intern_atom(conn: &impl Connection, name: &[u8]) -> anyhow::Result<Atom> {
    Ok(conn.intern_atom(false, name)?.reply()?.atom)
}

fn get_window_title(
    conn: &impl Connection,
    window: Window,
    net_wm_name: Atom,
    utf8_string: Atom,
) -> Option<String> {
    // Try _NET_WM_NAME (UTF-8) first
    let reply = conn
        .get_property(false, window, net_wm_name, utf8_string, 0, 1024)
        .ok()?
        .reply()
        .ok()?;
    if !reply.value.is_empty() {
        return Some(String::from_utf8_lossy(&reply.value).to_string());
    }

    // Fall back to WM_NAME
    let reply = conn
        .get_property(
            false,
            window,
            xproto::AtomEnum::WM_NAME,
            xproto::AtomEnum::STRING,
            0,
            1024,
        )
        .ok()?
        .reply()
        .ok()?;
    if !reply.value.is_empty() {
        return Some(String::from_utf8_lossy(&reply.value).to_string());
    }

    None
}

/// Returns (instance, class) from WM_CLASS, e.g. ("google-chrome", "Google-chrome")
fn get_wm_class_parts(conn: &impl Connection, window: Window) -> Option<(String, String)> {
    let reply = conn
        .get_property(
            false,
            window,
            xproto::AtomEnum::WM_CLASS,
            xproto::AtomEnum::STRING,
            0,
            1024,
        )
        .ok()?
        .reply()
        .ok()?;

    if reply.value.is_empty() {
        return None;
    }

    // WM_CLASS is two null-terminated strings: "instance\0class\0"
    let s = String::from_utf8_lossy(&reply.value);
    let mut parts = s.split('\0');
    let instance = parts.next().unwrap_or_default().to_string();
    let class = parts.next().unwrap_or_default().to_string();
    if instance.is_empty() && class.is_empty() {
        None
    } else {
        Some((instance, class))
    }
}

/// Turn a WM_CLASS name into something human-readable.
/// "com.mitchellh.ghostty" → "Ghostty", "dev.zed.Zed" → "Zed", "Firefox" → "Firefox"
fn friendly_name(class: &str) -> String {
    let base = if class.contains('.') {
        // Reverse-domain: take the last segment
        class.rsplit('.').next().unwrap_or(class)
    } else {
        class
    };
    // Capitalize first letter
    let mut chars = base.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => class.to_string(),
    }
}
