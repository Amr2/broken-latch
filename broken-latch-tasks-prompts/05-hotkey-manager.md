# Task 05: Hotkey Manager

**Platform: broken-latch**  
**Dependencies:** Task 01 (Project Setup), Task 04 (Game Lifecycle Detector)  
**Estimated Complexity:** Medium  
**Priority:** P0 (Critical Path)

---

## Objective

Implement the global hotkey registration system that allows apps to register Windows-level keyboard shortcuts (e.g., Alt+H, Ctrl+Shift+T) that work even when League of Legends has focus. The platform manages hotkey registration, conflict detection, and routes key presses to the correct app.

---

## Context

Global hotkeys are essential for app usability. Apps like Hunter Mode need to let users toggle panels while in-game without Alt+Tabbing. The platform:

- Uses Windows `RegisterHotKey` API for system-wide hotkeys
- Maintains a registry of which app owns which hotkey
- Prevents conflicts (two apps can't use the same hotkey)
- Stores hotkey bindings in the database for persistence
- Emits events to apps when their hotkeys are pressed

Apps register hotkeys via `LOLOverlay.hotkeys.register({ id: "toggle", keys: "Alt+H", onPress: fn })` in the JS SDK.

---

## What You Need to Build

### 1. Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
]}
```

### 2. Hotkey Data Model (`src-tauri/src/hotkey.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
    UI::Input::KeyboardAndMouse::*,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyRegistration {
    pub app_id: String,
    pub hotkey_id: String,     // App-defined ID (e.g. "toggle-main-panel")
    pub keys: String,          // Human-readable (e.g. "Alt+H")
    pub win_hotkey_id: i32,    // Windows RegisterHotKey ID
}

#[derive(Debug)]
pub struct HotkeyManager {
    registrations: Arc<Mutex<HashMap<i32, HotkeyRegistration>>>,
    next_id: Arc<Mutex<i32>>,
    message_window: HWND,
}

impl HotkeyManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create invisible message-only window for hotkey messages
        let message_window = Self::create_message_window()?;

        Ok(Self {
            registrations: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            message_window,
        })
    }

    /// Register a global hotkey
    pub fn register(
        &self,
        app_id: &str,
        hotkey_id: &str,
        keys: &str,
    ) -> Result<i32, HotkeyError> {
        // Check if hotkey is already registered
        if self.is_hotkey_taken(keys) {
            return Err(HotkeyError::AlreadyRegistered);
        }

        // Parse key string into modifiers + virtual key code
        let (modifiers, vk_code) = self.parse_keys(keys)?;

        // Get next available ID
        let mut next_id = self.next_id.lock().unwrap();
        let win_id = *next_id;
        *next_id += 1;

        // Register with Windows
        unsafe {
            let result = RegisterHotKey(
                self.message_window,
                win_id,
                modifiers,
                vk_code as u32,
            );

            if result.is_err() {
                return Err(HotkeyError::RegistrationFailed);
            }
        }

        // Store registration
        let registration = HotkeyRegistration {
            app_id: app_id.to_string(),
            hotkey_id: hotkey_id.to_string(),
            keys: keys.to_string(),
            win_hotkey_id: win_id,
        };

        self.registrations.lock().unwrap().insert(win_id, registration);

        log::info!("Registered hotkey {} for app {}: {}", hotkey_id, app_id, keys);

        Ok(win_id)
    }

    /// Unregister a hotkey
    pub fn unregister(&self, win_hotkey_id: i32) -> Result<(), HotkeyError> {
        unsafe {
            UnregisterHotKey(self.message_window, win_hotkey_id)
                .map_err(|_| HotkeyError::UnregistrationFailed)?;
        }

        self.registrations.lock().unwrap().remove(&win_hotkey_id);

        log::info!("Unregistered hotkey ID: {}", win_hotkey_id);

        Ok(())
    }

    /// Unregister all hotkeys for an app
    pub fn unregister_all_for_app(&self, app_id: &str) -> Result<(), HotkeyError> {
        let registrations = self.registrations.lock().unwrap();
        let ids_to_remove: Vec<i32> = registrations
            .iter()
            .filter(|(_, reg)| reg.app_id == app_id)
            .map(|(id, _)| *id)
            .collect();

        drop(registrations);

        for id in ids_to_remove {
            self.unregister(id)?;
        }

        Ok(())
    }

    /// Check if a key combination is already registered
    pub fn is_hotkey_taken(&self, keys: &str) -> bool {
        let registrations = self.registrations.lock().unwrap();
        registrations.values().any(|reg| reg.keys.eq_ignore_ascii_case(keys))
    }

    /// Get all hotkeys for an app
    pub fn get_hotkeys_for_app(&self, app_id: &str) -> Vec<HotkeyRegistration> {
        self.registrations
            .lock()
            .unwrap()
            .values()
            .filter(|reg| reg.app_id == app_id)
            .cloned()
            .collect()
    }

    /// Handle WM_HOTKEY message and return the registration
    pub fn handle_hotkey_message(&self, wparam: WPARAM) -> Option<HotkeyRegistration> {
        let hotkey_id = wparam.0 as i32;
        self.registrations.lock().unwrap().get(&hotkey_id).cloned()
    }

    /// Parse key string like "Alt+H" into Windows modifiers + virtual key
    fn parse_keys(&self, keys: &str) -> Result<(HOT_KEY_MODIFIERS, u16), HotkeyError> {
        let parts: Vec<&str> = keys.split('+').map(|s| s.trim()).collect();

        if parts.is_empty() {
            return Err(HotkeyError::InvalidKeyFormat);
        }

        let mut modifiers = HOT_KEY_MODIFIERS(0);
        let mut vk_code: Option<u16> = None;

        for part in parts {
            match part.to_uppercase().as_str() {
                "ALT" => modifiers |= MOD_ALT,
                "CTRL" | "CONTROL" => modifiers |= MOD_CONTROL,
                "SHIFT" => modifiers |= MOD_SHIFT,
                "WIN" | "WINDOWS" => modifiers |= MOD_WIN,
                key => {
                    // Parse the main key
                    vk_code = Some(self.key_to_vk(key)?);
                }
            }
        }

        let vk = vk_code.ok_or(HotkeyError::InvalidKeyFormat)?;
        Ok((modifiers, vk))
    }

    /// Convert key name to Windows virtual key code
    fn key_to_vk(&self, key: &str) -> Result<u16, HotkeyError> {
        let vk = match key.to_uppercase().as_str() {
            // Letters
            "A" => 0x41, "B" => 0x42, "C" => 0x43, "D" => 0x44,
            "E" => 0x45, "F" => 0x46, "G" => 0x47, "H" => 0x48,
            "I" => 0x49, "J" => 0x4A, "K" => 0x4B, "L" => 0x4C,
            "M" => 0x4D, "N" => 0x4E, "O" => 0x4F, "P" => 0x50,
            "Q" => 0x51, "R" => 0x52, "S" => 0x53, "T" => 0x54,
            "U" => 0x55, "V" => 0x56, "W" => 0x57, "X" => 0x58,
            "Y" => 0x59, "Z" => 0x5A,

            // Numbers
            "0" => 0x30, "1" => 0x31, "2" => 0x32, "3" => 0x33,
            "4" => 0x34, "5" => 0x35, "6" => 0x36, "7" => 0x37,
            "8" => 0x38, "9" => 0x39,

            // Function keys
            "F1" => VK_F1.0 as u16, "F2" => VK_F2.0 as u16,
            "F3" => VK_F3.0 as u16, "F4" => VK_F4.0 as u16,
            "F5" => VK_F5.0 as u16, "F6" => VK_F6.0 as u16,
            "F7" => VK_F7.0 as u16, "F8" => VK_F8.0 as u16,
            "F9" => VK_F9.0 as u16, "F10" => VK_F10.0 as u16,
            "F11" => VK_F11.0 as u16, "F12" => VK_F12.0 as u16,

            // Special keys
            "SPACE" => VK_SPACE.0 as u16,
            "TAB" => VK_TAB.0 as u16,
            "ESC" | "ESCAPE" => VK_ESCAPE.0 as u16,
            "ENTER" | "RETURN" => VK_RETURN.0 as u16,

            _ => return Err(HotkeyError::InvalidKeyFormat),
        };

        Ok(vk)
    }

    /// Create invisible message-only window to receive hotkey messages
    fn create_message_window() -> Result<HWND, Box<dyn std::error::Error>> {
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("broken-latch-hotkey-window"),
                WINDOW_STYLE(0),
                0, 0, 0, 0,
                HWND_MESSAGE,  // Message-only window
                None,
                None,
                None,
            )?;

            Ok(hwnd)
        }
    }

    /// Start message loop to listen for WM_HOTKEY messages
    pub fn start_message_loop(
        self: Arc<Self>,
        app_handle: tauri::AppHandle,
    ) {
        std::thread::spawn(move || {
            unsafe {
                let mut msg = MSG::default();

                loop {
                    let result = GetMessageW(&mut msg, self.message_window, 0, 0);

                    if result.is_err() || result.unwrap().0 == 0 {
                        break;
                    }

                    if msg.message == WM_HOTKEY {
                        if let Some(registration) = self.handle_hotkey_message(msg.wParam) {
                            log::debug!("Hotkey pressed: {} for app {}",
                                registration.hotkey_id, registration.app_id);

                            // Emit event to app
                            app_handle.emit_all(
                                &format!("hotkey_pressed:{}", registration.app_id),
                                &registration,
                            ).ok();
                        }
                    }

                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        });
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HotkeyError {
    #[error("Hotkey already registered")]
    AlreadyRegistered,

    #[error("Failed to register hotkey")]
    RegistrationFailed,

    #[error("Failed to unregister hotkey")]
    UnregistrationFailed,

    #[error("Invalid key format")]
    InvalidKeyFormat,
}
```

### 3. Database Persistence

Hotkeys are already defined in the database schema from Task 01. Add helper functions to save/load:

```rust
// Add to src-tauri/src/db.rs

pub async fn save_hotkey(
    app: &tauri::AppHandle,
    registration: &HotkeyRegistration,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = app.state::<Database>();

    db.execute(
        "INSERT OR REPLACE INTO hotkeys (app_id, hotkey_id, keys, win_hotkey_id, registered_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        &[
            &registration.app_id,
            &registration.hotkey_id,
            &registration.keys,
            &registration.win_hotkey_id.to_string(),
            &chrono::Utc::now().timestamp().to_string(),
        ],
    ).await?;

    Ok(())
}

pub async fn load_hotkeys_for_app(
    app: &tauri::AppHandle,
    app_id: &str,
) -> Result<Vec<HotkeyRegistration>, Box<dyn std::error::Error>> {
    let db = app.state::<Database>();

    let rows = db.query(
        "SELECT app_id, hotkey_id, keys, win_hotkey_id FROM hotkeys WHERE app_id = ?1",
        &[app_id],
    ).await?;

    // Parse rows into HotkeyRegistration structs
    // TODO: Implement row parsing

    Ok(vec![])
}

pub async fn delete_hotkey(
    app: &tauri::AppHandle,
    app_id: &str,
    hotkey_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = app.state::<Database>();

    db.execute(
        "DELETE FROM hotkeys WHERE app_id = ?1 AND hotkey_id = ?2",
        &[app_id, hotkey_id],
    ).await?;

    Ok(())
}
```

### 4. Tauri Commands (`src-tauri/src/main.rs`)

```rust
use crate::hotkey::{HotkeyManager, HotkeyRegistration};

#[tauri::command]
async fn register_hotkey(
    app_id: String,
    hotkey_id: String,
    keys: String,
    manager: tauri::State<'_, Arc<HotkeyManager>>,
    app: tauri::AppHandle,
) -> Result<i32, String> {
    let win_id = manager.register(&app_id, &hotkey_id, &keys)
        .map_err(|e| e.to_string())?;

    // Save to database
    let registration = HotkeyRegistration {
        app_id,
        hotkey_id,
        keys,
        win_hotkey_id: win_id,
    };

    crate::db::save_hotkey(&app, &registration).await
        .map_err(|e| e.to_string())?;

    Ok(win_id)
}

#[tauri::command]
async fn unregister_hotkey(
    win_hotkey_id: i32,
    manager: tauri::State<'_, Arc<HotkeyManager>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    manager.unregister(win_hotkey_id)
        .map_err(|e| e.to_string())?;

    // Remove from database
    // TODO: Implement database deletion by win_hotkey_id

    Ok(())
}

#[tauri::command]
async fn is_hotkey_registered(
    keys: String,
    manager: tauri::State<'_, Arc<HotkeyManager>>,
) -> Result<bool, String> {
    Ok(manager.is_hotkey_taken(&keys))
}

#[tauri::command]
async fn get_app_hotkeys(
    app_id: String,
    manager: tauri::State<'_, Arc<HotkeyManager>>,
) -> Result<Vec<HotkeyRegistration>, String> {
    Ok(manager.get_hotkeys_for_app(&app_id))
}

// In main():
fn main() {
    let hotkey_manager = Arc::new(HotkeyManager::new().unwrap());
    let hotkey_manager_clone = hotkey_manager.clone();

    tauri::Builder::default()
        .manage(hotkey_manager)
        .invoke_handler(tauri::generate_handler![
            register_hotkey,
            unregister_hotkey,
            is_hotkey_registered,
            get_app_hotkeys,
        ])
        .setup(|app| {
            let app_handle = app.handle();

            // Start hotkey message loop
            hotkey_manager_clone.start_message_loop(app_handle);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Integration Points

### Integration with Task 04 (Game Lifecycle Detector)

- Hotkeys only active when game phase is `InGame` or `Loading` (optional)
- Can disable hotkeys when game is not running to avoid conflicts

### Integration with Task 07 (App Lifecycle Manager)

- When app is stopped, all its hotkeys are unregistered
- When app starts, hotkeys from manifest are auto-registered
- `hotkeys` field in app manifest is processed by Task 07

### Integration with Task 08 (HTTP API Server)

- Exposes `POST /api/hotkeys/register`, `POST /api/hotkeys/unregister` endpoints
- Broadcasts `app_hotkey_pressed` events to app webhooks

### Integration with Task 09 (JavaScript SDK)

- SDK provides `LOLOverlay.hotkeys.register()` wrapper
- SDK emits `onHotkeyPress` events to app listeners

---

## Testing Requirements

### Unit Tests

Create `src-tauri/src/hotkey.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_hotkey() {
        let manager = HotkeyManager::new().unwrap();
        let (modifiers, vk) = manager.parse_keys("Alt+H").unwrap();

        assert_eq!(modifiers.0 & MOD_ALT.0, MOD_ALT.0);
        assert_eq!(vk, 0x48); // 'H' key
    }

    #[test]
    fn test_parse_multi_modifier_hotkey() {
        let manager = HotkeyManager::new().unwrap();
        let (modifiers, vk) = manager.parse_keys("Ctrl+Shift+F").unwrap();

        assert_eq!(modifiers.0 & MOD_CONTROL.0, MOD_CONTROL.0);
        assert_eq!(modifiers.0 & MOD_SHIFT.0, MOD_SHIFT.0);
        assert_eq!(vk, 0x46); // 'F' key
    }

    #[test]
    fn test_key_to_vk() {
        let manager = HotkeyManager::new().unwrap();

        assert_eq!(manager.key_to_vk("A").unwrap(), 0x41);
        assert_eq!(manager.key_to_vk("Z").unwrap(), 0x5A);
        assert_eq!(manager.key_to_vk("0").unwrap(), 0x30);
        assert_eq!(manager.key_to_vk("F1").unwrap(), VK_F1.0 as u16);
    }

    #[test]
    fn test_invalid_key_format() {
        let manager = HotkeyManager::new().unwrap();
        assert!(manager.parse_keys("InvalidKey").is_err());
        assert!(manager.parse_keys("Alt+").is_err());
    }

    #[test]
    fn test_hotkey_conflict_detection() {
        let manager = HotkeyManager::new().unwrap();

        manager.register("app1", "toggle", "Alt+H").unwrap();

        assert!(manager.is_hotkey_taken("Alt+H"));
        assert!(!manager.is_hotkey_taken("Alt+J"));
    }
}
```

### Integration Tests

Create `src-tauri/tests/hotkey_test.rs`:

```rust
#[tokio::test]
async fn test_hotkey_registration_flow() {
    let manager = Arc::new(HotkeyManager::new().unwrap());

    // Register hotkey
    let win_id = manager.register("test-app", "toggle", "Alt+T").unwrap();
    assert!(win_id > 0);

    // Verify registration
    let hotkeys = manager.get_hotkeys_for_app("test-app");
    assert_eq!(hotkeys.len(), 1);
    assert_eq!(hotkeys[0].keys, "Alt+T");

    // Unregister
    manager.unregister(win_id).unwrap();

    // Verify unregistration
    let hotkeys = manager.get_hotkeys_for_app("test-app");
    assert_eq!(hotkeys.len(), 0);
}
```

### Manual Testing Checklist

- [ ] Register hotkey via Tauri command succeeds
- [ ] Pressing registered hotkey triggers event
- [ ] Hotkey works even when League of Legends has focus
- [ ] Conflicting hotkey registration returns error
- [ ] Unregistering hotkey stops it from triggering
- [ ] Multiple apps can register different hotkeys
- [ ] Hotkeys persist across platform restarts (loaded from DB)
- [ ] Message loop runs without crashing
- [ ] Hotkey event contains correct app_id and hotkey_id
- [ ] Reserved platform hotkeys (Alt+Shift+L) cannot be registered by apps

---

## Acceptance Criteria

✅ **Complete when:**

1. HotkeyManager compiles and initializes successfully
2. Global hotkeys can be registered and unregistered
3. Hotkey presses trigger Tauri events with registration data
4. Key string parsing supports all common modifiers and keys
5. Conflict detection prevents duplicate hotkey registration
6. Hotkeys are saved to and loaded from database
7. Message loop runs continuously without blocking main thread
8. All unit tests pass
9. Integration tests pass
10. Manual testing checklist is 100% complete
11. Hotkeys work while League of Legends has focus
12. No memory leaks in message loop

---

## Dependencies for Next Tasks

- **Task 06** depends on: Hotkey events to toggle widget visibility
- **Task 07** depends on: Hotkey registration from app manifests
- **Task 08** depends on: Hotkey commands for HTTP API
- **Task 09** depends on: Tauri commands for SDK wrapping

---

## Files to Create/Modify

### New Files:

- `src-tauri/src/hotkey.rs`
- `src-tauri/tests/hotkey_test.rs`

### Modified Files:

- `src-tauri/src/main.rs` (add commands and start message loop)
- `src-tauri/src/db.rs` (add hotkey persistence functions)
- `src-tauri/Cargo.toml` (add windows dependency)

---

## Expected Time: 6-8 hours

## Difficulty: Medium (Windows API + message loops)
