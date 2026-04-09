//! PhaseManager — drives window/overlay lifecycle across game phase transitions.
//!
//! # How it works
//!
//! Each game phase is declared in `src-tauri/resources/phases.json`.
//! A phase entry lists zero or more *windows* (normal decorated Tauri windows)
//! and zero or more *overlays* (transparent fullscreen Win32 overlay windows).
//!
//! When `transition(new_phase)` is called:
//!   1. Windows/overlays present in the old phase but absent from the new one
//!      are closed.
//!   2. Windows/overlays present in the new phase but not already open are
//!      created.
//!   3. Windows/overlays whose `id` appears in BOTH phases are kept open
//!      without interruption — useful for persistent widgets.
//!
//! # Window labels
//! All phase-managed windows get the Tauri label `phase-{id}` so the
//! `phase-windows` capability covers them with a single glob pattern.
//!
//! # URL resolution
//! If `url` starts with `http://` or `https://`, a Tauri External WebviewUrl
//! is used (opens the URL directly).  Otherwise the value is treated as a path
//! to a bundled HTML page (e.g. `ingame-overlay.html`).

use std::collections::{HashMap, HashSet};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, WebviewWindow};
use crate::overlay::{OverlayWindow, make_webview_url};

// ---------------------------------------------------------------------------
// Config types  (deserialized from phases.json)
// ---------------------------------------------------------------------------

/// Configuration for a normal (decorated or borderless) Tauri window.
#[derive(Debug, Clone, Deserialize)]
pub struct WindowConfig {
    /// Unique identifier — also becomes the Tauri window label `phase-{id}`.
    pub id: String,
    /// Bundled HTML page name or full external URL.
    pub url: String,
    /// Window title bar text.
    #[serde(default)]
    pub title: String,
    pub width: u32,
    pub height: u32,
    /// Pixel position from top-left.  `null` → centre on primary monitor.
    pub x: Option<i32>,
    pub y: Option<i32>,
    /// Show OS window chrome (title bar, borders).
    pub decorations: bool,
    /// Keep above other windows.
    pub always_on_top: bool,
    pub resizable: bool,
}

/// Configuration for a transparent fullscreen overlay window.
#[derive(Debug, Clone, Deserialize)]
pub struct OverlayConfig {
    /// Unique identifier — becomes the Tauri window label `phase-{id}`.
    pub id: String,
    /// Bundled HTML page name or full external URL.
    pub url: String,
    /// Initial opacity 0.0 (invisible) – 1.0 (opaque).
    pub opacity: f32,
    /// When `false` (default) the overlay is hidden from OBS/Discord/Game Bar.
    pub screen_capture_visible: bool,
}

/// All windows + overlays for a single phase.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PhaseConfig {
    #[serde(default)]
    pub windows: Vec<WindowConfig>,
    #[serde(default)]
    pub overlays: Vec<OverlayConfig>,
}

// ---------------------------------------------------------------------------
// Internal window tracking
// ---------------------------------------------------------------------------

enum ManagedWindow {
    Normal(WebviewWindow),
    Overlay(OverlayWindow),
}

impl ManagedWindow {
    fn close(&self) {
        match self {
            ManagedWindow::Normal(w)  => { let _ = w.close(); }
            ManagedWindow::Overlay(o) => { let _ = o.close(); }
        }
    }
}

// ---------------------------------------------------------------------------
// PhaseManager
// ---------------------------------------------------------------------------

pub struct PhaseManager {
    /// Parsed contents of phases.json.
    phases: HashMap<String, PhaseConfig>,
    /// All currently open phase windows, keyed by their config `id`.
    open: HashMap<String, ManagedWindow>,
    /// Active phase string.
    current_phase: String,
}

impl PhaseManager {
    /// Construct from the raw JSON content of `phases.json`.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let phases: HashMap<String, PhaseConfig> = serde_json::from_str(json)?;
        Ok(Self {
            phases,
            open: HashMap::new(),
            current_phase: "NOT_RUNNING".to_string(),
        })
    }

    pub fn current_phase(&self) -> &str {
        &self.current_phase
    }

    /// Transition to `new_phase`.
    ///
    /// Closes windows/overlays no longer needed, opens new ones, leaves shared
    /// ones (same `id` in both phases) untouched.
    /// Emits the `phase-changed` Tauri event after the transition.
    pub fn transition(
        &mut self,
        new_phase: &str,
        app: &AppHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_phase == new_phase {
            return Ok(());
        }

        let empty = PhaseConfig::default();
        let new_cfg = self.phases.get(new_phase).cloned().unwrap_or_else(|| {
            eprintln!("[phase] unknown phase '{}', treating as empty", new_phase);
            empty.clone()
        });
        let old_cfg = self.phases.get(&self.current_phase).cloned().unwrap_or(empty);

        // IDs required in new phase
        let new_ids: HashSet<String> = new_cfg.windows.iter().map(|w| w.id.clone())
            .chain(new_cfg.overlays.iter().map(|o| o.id.clone()))
            .collect();

        // IDs present in old phase
        let old_ids: HashSet<String> = old_cfg.windows.iter().map(|w| w.id.clone())
            .chain(old_cfg.overlays.iter().map(|o| o.id.clone()))
            .collect();

        // Close windows that are going away
        for id in old_ids.difference(&new_ids) {
            if let Some(win) = self.open.remove(id) {
                win.close();
                println!("[phase] closed  '{}'", id);
            }
        }

        // Open new windows
        for win_cfg in &new_cfg.windows {
            if !self.open.contains_key(&win_cfg.id) {
                match Self::open_window(app, win_cfg) {
                    Ok(w) => {
                        println!("[phase] opened window  '{}'", win_cfg.id);
                        self.open.insert(win_cfg.id.clone(), ManagedWindow::Normal(w));
                    }
                    Err(e) => eprintln!("[phase] window '{}' open error: {}", win_cfg.id, e),
                }
            }
        }

        // Open new overlays
        for ov_cfg in &new_cfg.overlays {
            if !self.open.contains_key(&ov_cfg.id) {
                match Self::open_overlay(app, ov_cfg) {
                    Ok(o) => {
                        println!("[phase] opened overlay '{}'", ov_cfg.id);
                        self.open.insert(ov_cfg.id.clone(), ManagedWindow::Overlay(o));
                    }
                    Err(e) => eprintln!("[phase] overlay '{}' open error: {}", ov_cfg.id, e),
                }
            }
        }

        self.current_phase = new_phase.to_string();
        let _ = app.emit("phase-changed", new_phase);
        println!("[phase] → {}", new_phase);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Overlay control helpers (used by Tauri commands)
    // -----------------------------------------------------------------------

    /// Return the first active overlay window (for opacity/region commands).
    pub fn first_overlay(&mut self) -> Option<&mut OverlayWindow> {
        for win in self.open.values_mut() {
            if let ManagedWindow::Overlay(ov) = win {
                return Some(ov);
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Private constructors
    // -----------------------------------------------------------------------

    fn open_window(app: &AppHandle, cfg: &WindowConfig) -> Result<WebviewWindow, Box<dyn std::error::Error>> {
        let label = format!("phase-{}", cfg.id);
        let url   = make_webview_url(&cfg.url);

        let title = if cfg.title.is_empty() { cfg.id.clone() } else { cfg.title.clone() };

        let mut builder = tauri::WebviewWindowBuilder::new(app, &label, url)
            .title(title)
            .inner_size(cfg.width as f64, cfg.height as f64)
            .decorations(cfg.decorations)
            .always_on_top(cfg.always_on_top)
            .resizable(cfg.resizable);

        builder = if let (Some(x), Some(y)) = (cfg.x, cfg.y) {
            builder.position(x as f64, y as f64)
        } else {
            builder.center()
        };

        Ok(builder.build()?)
    }

    fn open_overlay(app: &AppHandle, cfg: &OverlayConfig) -> Result<OverlayWindow, Box<dyn std::error::Error>> {
        let label = format!("phase-{}", cfg.id);
        OverlayWindow::create(app, &label, &cfg.url, cfg.opacity, cfg.screen_capture_visible)
    }
}
