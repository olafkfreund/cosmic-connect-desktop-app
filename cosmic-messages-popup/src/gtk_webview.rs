//! GTK WebView Window Module
//!
//! Creates standalone GTK windows with embedded WebView for Wayland compatibility.
//! This approach works on both X11 and Wayland, unlike the embedded window handle approach.
//!
//! Note: GTK operations must happen on the main GTK thread. This module provides
//! a channel-based API for cross-thread communication.

use crate::config::Config;
use crate::webview::user_agent_for_messenger;
use anyhow::{Context, Result};
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, info};
use wry::WebViewBuilderExtUnix;
use wry::{WebView, WebViewBuilder};

/// Global GTK initialization flag
static GTK_INITIALIZED: OnceLock<bool> = OnceLock::new();

/// Commands that can be sent to the GTK thread
#[derive(Debug, Clone)]
pub enum GtkCommand {
    /// Show a WebView window for a messenger
    Show {
        messenger_id: String,
        url: String,
        title: String,
        width: i32,
        height: i32,
        position: String,
    },
    /// Hide a WebView window
    Hide { messenger_id: String },
    /// Hide all windows
    HideAll,
    /// Navigate to a URL
    Navigate { messenger_id: String, url: String },
    /// Reload a WebView window
    Reload { messenger_id: String },
    /// Close a window
    Close { messenger_id: String },
    /// Shutdown the GTK thread
    Shutdown,
}

/// Channel sender for GTK commands
static GTK_SENDER: OnceLock<Sender<GtkCommand>> = OnceLock::new();

/// Initialize GTK if not already initialized
#[allow(dead_code)]
pub fn ensure_gtk_init() -> Result<()> {
    let initialized = GTK_INITIALIZED.get_or_init(|| match gtk::init() {
        Ok(()) => {
            info!("GTK initialized successfully");
            true
        }
        Err(e) => {
            error!("Failed to initialize GTK: {}", e);
            false
        }
    });

    if *initialized {
        Ok(())
    } else {
        anyhow::bail!("GTK initialization failed")
    }
}

/// Handle a GTK command
fn handle_gtk_command(
    cmd: GtkCommand,
    windows: &mut HashMap<String, (gtk::Window, WebView, wry::WebContext)>,
) {
    match cmd {
        GtkCommand::Show {
            messenger_id,
            url,
            title,
            width,
            height,
            position,
        } => {
            if let Some((window, _, _)) = windows.get(&messenger_id) {
                // Window exists, just show it
                window.present();
                window.grab_focus();
                debug!("Presenting existing window for {}", messenger_id);
            } else {
                // Create new window
                match create_webview_window(&messenger_id, &url, &title, width, height, &position) {
                    Ok((window, webview, context)) => {
                        windows.insert(messenger_id.clone(), (window, webview, context));
                        info!("Created WebView window for {}", messenger_id);
                    }
                    Err(e) => {
                        error!("Failed to create WebView window: {}", e);
                    }
                }
            }
        }
        GtkCommand::Hide { messenger_id } => {
            if let Some((window, _, _)) = windows.get(&messenger_id) {
                window.hide();
                debug!("Hidden window for {}", messenger_id);
            }
        }
        GtkCommand::HideAll => {
            for (window, _, _) in windows.values() {
                window.hide();
            }
            debug!("Hidden all windows");
        }
        GtkCommand::Navigate { messenger_id, url } => {
            if let Some((_, webview, _)) = windows.get(&messenger_id) {
                if let Err(e) = webview.load_url(&url) {
                    error!("Failed to navigate: {}", e);
                }
                debug!("Navigated {} to {}", messenger_id, url);
            }
        }
        GtkCommand::Reload { messenger_id } => {
            if let Some((_, webview, _)) = windows.get(&messenger_id) {
                if let Err(e) = webview.evaluate_script("window.location.reload()") {
                    error!("Failed to reload: {}", e);
                }
                debug!("Reloaded {}", messenger_id);
            }
        }
        GtkCommand::Close { messenger_id } => {
            if let Some((window, _, _)) = windows.remove(&messenger_id) {
                window.close();
                info!("Closed window for {}", messenger_id);
            }
        }
        GtkCommand::Shutdown => {
            info!("GTK thread shutting down");
            for (_, (window, _, _)) in windows.drain() {
                window.close();
            }
            gtk::main_quit();
        }
    }
}

/// Start the GTK event loop in a background thread
///
/// Returns the thread handle
///
/// NOTE: GTK must be initialized ON the thread where it will be used.
/// This function initializes GTK inside the spawned thread.
pub fn start_gtk_event_loop() -> JoinHandle<()> {
    let (tx, rx): (Sender<GtkCommand>, Receiver<GtkCommand>) = mpsc::channel();

    // Store sender globally
    let _ = GTK_SENDER.set(tx);

    thread::spawn(move || {
        // Initialize GTK on THIS thread (GTK requires all ops on same thread)
        if let Err(e) = gtk::init() {
            error!("Failed to initialize GTK on event loop thread: {}", e);
            return;
        }
        info!("GTK initialized on event loop thread");
        info!("GTK event loop thread started");

        // Track windows by messenger ID (with WebContext to keep it alive)
        type WindowMap = HashMap<String, (gtk::Window, WebView, wry::WebContext)>;
        let windows: Rc<RefCell<WindowMap>> = Rc::new(RefCell::new(HashMap::new()));

        // Poll commands via glib timeout (GTK native event loop integration)
        let windows_clone = windows.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            match rx.try_recv() {
                Ok(cmd) => {
                    handle_gtk_command(cmd, &mut windows_clone.borrow_mut());
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No commands, continue
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    info!("GTK command channel disconnected, shutting down");
                    gtk::main_quit();
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Continue
        });

        // Run GTK main loop (proper event dispatch)
        gtk::main();
        info!("GTK main loop exited");
    })
}

/// Create a GTK window with embedded WebView
fn create_webview_window(
    messenger_id: &str,
    url: &str,
    title: &str,
    width: i32,
    height: i32,
    position: &str,
) -> Result<(gtk::Window, WebView, wry::WebContext)> {
    // Create persistent data directory for this messenger's sessions
    // This stores cookies, local storage, IndexedDB - users only login once!
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cosmic-messages-popup")
        .join("webview-data")
        .join(messenger_id);

    // Ensure the directory exists
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        error!("Failed to create WebView data directory: {}", e);
    }
    info!(
        "WebView data directory for {}: {:?}",
        messenger_id, data_dir
    );

    // Create GTK window
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title(title);
    window.set_default_size(width, height);

    // Apply positioning
    match position {
        "center" => window.set_position(gtk::WindowPosition::Center),
        "cursor" => window.set_position(gtk::WindowPosition::Mouse),
        "bottom-right" => {
            window.set_position(gtk::WindowPosition::None);
            window.set_gravity(gdk::Gravity::SouthEast);
        }
        _ => window.set_position(gtk::WindowPosition::Center),
    }

    // Set window hints for popup-like behavior
    window.set_type_hint(gdk::WindowTypeHint::Utility);
    window.set_decorated(true);
    window.set_resizable(true);

    // Create a Box container that expands to fill the window
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.set_hexpand(true);
    container.set_vexpand(true);
    window.add(&container);

    // Create a WebContext with persistent data directory
    // This stores cookies, local storage, IndexedDB - users login once!
    let mut web_context = wry::WebContext::new(Some(data_dir.clone()));

    // Get user agent for this messenger
    let user_agent = user_agent_for_messenger(messenger_id);

    // Build WebView using GTK extension for Wayland support
    // Note: Don't use with_bounds() - let GTK handle sizing through widget properties
    let webview = WebViewBuilder::with_web_context(&mut web_context)
        .with_url(url)
        .with_user_agent(&user_agent)
        .with_devtools(cfg!(debug_assertions))
        .with_autoplay(true)
        // Handle new window requests (OAuth popups, etc.)
        .with_new_window_req_handler(|uri: String| {
            debug!("WebView requested new window: {}", uri);
            // Open OAuth/external links in default browser
            if uri.contains("accounts.google.com")
                || uri.contains("login.microsoftonline.com")
                || uri.contains("facebook.com/login")
                || uri.contains("facebook.com/v")
                || uri.contains("appleid.apple.com")
                || uri.contains("/oauth")
                || uri.contains("/login")
                || uri.contains("/signin")
                || uri.contains("/auth/")
                || uri.contains("/sso")
                || uri.starts_with("https://accounts.")
            {
                let _ = open::that(&uri);
                return false; // Don't open in webview
            }
            true
        })
        // Handle navigation requests
        .with_navigation_handler(move |uri: String| {
            debug!("WebView navigating to: {}", uri);
            true // Allow navigation
        })
        .build_gtk(&container)
        .context("Failed to build WebView")?;

    // Set all children of the container to expand and fill
    // The wry WebView adds a widget that needs to expand
    for child in container.children() {
        child.set_hexpand(true);
        child.set_vexpand(true);
        // If it's a Box, also set the child packing
        if let Some(parent_box) = child.parent().and_then(|p| p.downcast::<gtk::Box>().ok()) {
            parent_box.set_child_packing(&child, true, true, 0, gtk::PackType::Start);
        }
    }

    // Handle window close - hide instead of destroy
    let messenger_id_clone = messenger_id.to_string();
    window.connect_delete_event(move |win, _| {
        debug!("Window close requested for {}", messenger_id_clone);
        win.hide();
        glib::Propagation::Stop
    });

    // Show the window and grab focus
    window.show_all();
    window.present();
    window.grab_focus();

    info!("Created GTK WebView window for {} at {}", messenger_id, url);

    Ok((window, webview, web_context))
}

/// Send a command to the GTK thread
pub fn send_gtk_command(cmd: GtkCommand) -> Result<()> {
    let sender = GTK_SENDER
        .get()
        .ok_or_else(|| anyhow::anyhow!("GTK thread not started"))?;

    sender
        .send(cmd)
        .context("Failed to send command to GTK thread")
}

/// Show a messenger's WebView window
pub fn show_messenger_window(messenger_id: &str, url: &str, config: &Config) -> Result<()> {
    let display_name = config
        .enabled_messengers
        .iter()
        .find(|m| m.id == messenger_id)
        .map(|m| m.name.as_str())
        .unwrap_or(messenger_id);

    send_gtk_command(GtkCommand::Show {
        messenger_id: messenger_id.to_string(),
        url: url.to_string(),
        title: format!("{} - COSMIC Messages", display_name),
        width: config.popup.width as i32,
        height: config.popup.height as i32,
        position: config.popup.position.as_str().to_string(),
    })
}

/// Hide a messenger's WebView window
#[allow(dead_code)]
pub fn hide_messenger_window(messenger_id: &str) -> Result<()> {
    send_gtk_command(GtkCommand::Hide {
        messenger_id: messenger_id.to_string(),
    })
}

/// Hide all WebView windows
pub fn hide_all_windows() -> Result<()> {
    send_gtk_command(GtkCommand::HideAll)
}

/// Navigate a messenger's WebView to a new URL
#[allow(dead_code)]
pub fn navigate_messenger(messenger_id: &str, url: &str) -> Result<()> {
    send_gtk_command(GtkCommand::Navigate {
        messenger_id: messenger_id.to_string(),
        url: url.to_string(),
    })
}

/// Reload a messenger's WebView
#[allow(dead_code)]
pub fn reload_messenger(messenger_id: &str) -> Result<()> {
    send_gtk_command(GtkCommand::Reload {
        messenger_id: messenger_id.to_string(),
    })
}

/// Close a messenger's WebView window
#[allow(dead_code)]
pub fn close_messenger_window(messenger_id: &str) -> Result<()> {
    send_gtk_command(GtkCommand::Close {
        messenger_id: messenger_id.to_string(),
    })
}

/// Shutdown the GTK event loop
#[allow(dead_code)]
pub fn shutdown_gtk() -> Result<()> {
    send_gtk_command(GtkCommand::Shutdown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtk_command_variants() {
        // Test that commands can be created
        let _cmd = GtkCommand::Show {
            messenger_id: "test".to_string(),
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
            width: 400,
            height: 600,
            position: "center".to_string(),
        };

        let _cmd = GtkCommand::Hide {
            messenger_id: "test".to_string(),
        };

        let _cmd = GtkCommand::HideAll;
        let _cmd = GtkCommand::Reload {
            messenger_id: "test".to_string(),
        };
        let _cmd = GtkCommand::Shutdown;
    }
}
