//! PhaseManager — drives window/overlay lifecycle across game phase transitions.
//!
//! Each phase in `phases.json` owns N independent windows and M independent
//! overlays.  Every entry becomes a separate OS window.  On transition the
//! manager diffs old vs new config: closes stale windows, opens new ones,
//! leaves windows whose `id` appears in both phases untouched.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, WebviewWindow};
use crate::overlay::{OverlayWindow, make_webview_url};

// ---------------------------------------------------------------------------
// Config types  (phases.json schema)
// ---------------------------------------------------------------------------

fn default_true()     -> bool { true }
fn default_opacity()  -> f32  { 1.0  }

/// Configuration for a normal OS window.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfig {
    // --- identity ---
    pub id:    String,
    pub url:   String,
    #[serde(default)]
    pub title: String,

    // --- size & position ---
    pub width:  u32,
    pub height: u32,
    pub x: Option<i32>,   // null → OS centres the window
    pub y: Option<i32>,
    pub min_width:  Option<u32>,
    pub min_height: Option<u32>,

    // --- behaviour ---
    #[serde(default = "default_true")]
    pub decorations: bool,
    #[serde(default)]                 // windows default to NOT topmost
    pub always_on_top: bool,
    #[serde(default = "default_true")]
    pub resizable: bool,
    #[serde(default)]
    pub skip_taskbar: bool,
    #[serde(default = "default_true")]
    pub focused: bool,
}

/// Configuration for an independent overlay OS window.
/// Each entry = its own OS window (borderless, transparent, always-on-top by default).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OverlayConfig {
    // --- identity ---
    pub id:  String,
    pub url: String,

    // --- size & position ---
    // null width + null height → fullscreen (covers entire primary monitor)
    pub width:  Option<u32>,
    pub height: Option<u32>,
    pub x: Option<i32>,   // null + fullscreen → 0,0 ; null + sized → centred
    pub y: Option<i32>,

    // --- visual ---
    #[serde(default = "default_opacity")]
    pub opacity: f32,

    // --- behaviour ---
    #[serde(default = "default_true")]
    pub click_through: bool,            // WS_EX_TRANSPARENT  (default: true)
    #[serde(default = "default_true")]
    pub always_on_top: bool,            // HWND_TOPMOST       (default: true)
    #[serde(default)]
    pub screen_capture_visible: bool,   // WDA_EXCLUDEFROMCAPTURE inverse (default: false → hidden)

    // --- drag edge ---
    // Platform injects a 4 px drag strip at the top of the overlay.
    // Only interactive when click_through = false.
    // Developers can also call getCurrentWindow().startDragging() from any element.
    #[serde(default = "default_true")]
    pub drag_edge: bool,
}

/// All windows + overlays for a single phase.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PhaseConfig {
    #[serde(default)]
    pub windows:  Vec<WindowConfig>,
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
    phases:       HashMap<String, PhaseConfig>,
    /// Flat index: overlay id → config (for get_overlay_config command).
    all_overlays: HashMap<String, OverlayConfig>,
    open:         HashMap<String, ManagedWindow>,
    current_phase: String,
}

impl PhaseManager {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let phases: HashMap<String, PhaseConfig> = serde_json::from_str(json)?;

        // Build flat overlay lookup
        let mut all_overlays = HashMap::new();
        for cfg in phases.values() {
            for ov in &cfg.overlays {
                all_overlays.insert(ov.id.clone(), ov.clone());
            }
        }

        Ok(Self {
            phases,
            all_overlays,
            open: HashMap::new(),
            current_phase: "NOT_RUNNING".to_string(),
        })
    }

    pub fn current_phase(&self) -> &str {
        &self.current_phase
    }

    /// Look up an overlay's config by its id (used by `get_overlay_config` command).
    pub fn get_overlay_config(&self, id: &str) -> Option<&OverlayConfig> {
        self.all_overlays.get(id)
    }

    // -----------------------------------------------------------------------
    // Phase transition
    // -----------------------------------------------------------------------

    pub fn transition(
        &mut self,
        new_phase: &str,
        app: &AppHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_phase == new_phase { return Ok(()); }

        let empty    = PhaseConfig::default();
        let new_cfg  = self.phases.get(new_phase).cloned().unwrap_or_else(|| {
            eprintln!("[phase] unknown phase '{}', treating as empty", new_phase);
            empty.clone()
        });
        let old_cfg  = self.phases.get(&self.current_phase).cloned().unwrap_or(empty);

        let new_ids: HashSet<String> = new_cfg.windows.iter().map(|w| w.id.clone())
            .chain(new_cfg.overlays.iter().map(|o| o.id.clone()))
            .collect();

        let old_ids: HashSet<String> = old_cfg.windows.iter().map(|w| w.id.clone())
            .chain(old_cfg.overlays.iter().map(|o| o.id.clone()))
            .collect();

        // Close windows no longer needed
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
                    Err(e) => eprintln!("[phase] window '{}' error: {}", win_cfg.id, e),
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
                    Err(e) => eprintln!("[phase] overlay '{}' error: {}", ov_cfg.id, e),
                }
            }
        }

        self.current_phase = new_phase.to_string();
        let _ = app.emit("phase-changed", new_phase);
        println!("[phase] → {}", new_phase);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Overlay helpers (for Tauri commands)
    // -----------------------------------------------------------------------

    pub fn first_overlay(&mut self) -> Option<&mut OverlayWindow> {
        for win in self.open.values_mut() {
            if let ManagedWindow::Overlay(ov) = win { return Some(ov); }
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

        let mut b = tauri::WebviewWindowBuilder::new(app, &label, url)
            .title(title)
            .inner_size(cfg.width as f64, cfg.height as f64)
            .decorations(cfg.decorations)
            .always_on_top(cfg.always_on_top)
            .resizable(cfg.resizable)
            .skip_taskbar(cfg.skip_taskbar)
            .focused(cfg.focused);

        if let (Some(mw), Some(mh)) = (cfg.min_width, cfg.min_height) {
            b = b.min_inner_size(mw as f64, mh as f64);
        } else if let Some(mw) = cfg.min_width {
            b = b.min_inner_size(mw as f64, 0.0);
        } else if let Some(mh) = cfg.min_height {
            b = b.min_inner_size(0.0, mh as f64);
        }

        b = if let (Some(x), Some(y)) = (cfg.x, cfg.y) {
            b.position(x as f64, y as f64)
        } else {
            b.center()
        };

        Ok(b.build()?)
    }

    fn open_overlay(app: &AppHandle, cfg: &OverlayConfig) -> Result<OverlayWindow, Box<dyn std::error::Error>> {
        let label = format!("phase-{}", cfg.id);
        OverlayWindow::create(
            app, &label, &cfg.url,
            cfg.width, cfg.height, cfg.x, cfg.y,
            cfg.opacity, cfg.click_through, cfg.always_on_top, cfg.screen_capture_visible,
        )
    }
}
