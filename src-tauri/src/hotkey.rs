use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use windows::core::w;
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
    UI::Input::KeyboardAndMouse::*,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyRegistration {
    pub app_id: String,
    pub hotkey_id: String,
    pub keys: String,
    pub win_hotkey_id: i32,
}

#[derive(Debug)]
pub struct HotkeyManager {
    registrations: Arc<Mutex<HashMap<i32, HotkeyRegistration>>>,
    next_id: Arc<Mutex<i32>>,
    // Store HWND as isize so HotkeyManager is Send+Sync
    message_window: isize,
}

// SAFETY: The message_window HWND (stored as isize) is only used from the
// message loop thread and Win32 registration calls, which are synchronized via Mutex.
unsafe impl Send for HotkeyManager {}
unsafe impl Sync for HotkeyManager {}

impl HotkeyManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let message_window = Self::create_message_window()?;

        Ok(Self {
            registrations: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            message_window: message_window.0 as isize,
        })
    }

    fn hwnd(&self) -> HWND {
        HWND(self.message_window as *mut _)
    }

    /// Register a global hotkey
    pub fn register(
        &self,
        app_id: &str,
        hotkey_id: &str,
        keys: &str,
    ) -> Result<i32, HotkeyError> {
        if self.is_hotkey_taken(keys) {
            return Err(HotkeyError::AlreadyRegistered);
        }

        let (modifiers, vk_code) = Self::parse_keys(keys)?;

        let mut next_id = self.next_id.lock().unwrap();
        let win_id = *next_id;
        *next_id += 1;

        unsafe {
            RegisterHotKey(Some(self.hwnd()), win_id, modifiers, vk_code as u32)
                .map_err(|_| HotkeyError::RegistrationFailed)?;
        }

        let registration = HotkeyRegistration {
            app_id: app_id.to_string(),
            hotkey_id: hotkey_id.to_string(),
            keys: keys.to_string(),
            win_hotkey_id: win_id,
        };

        self.registrations
            .lock()
            .unwrap()
            .insert(win_id, registration);

        log::info!(
            "Registered hotkey {} for app {}: {}",
            hotkey_id,
            app_id,
            keys
        );

        Ok(win_id)
    }

    /// Unregister a hotkey
    pub fn unregister(&self, win_hotkey_id: i32) -> Result<(), HotkeyError> {
        unsafe {
            UnregisterHotKey(Some(self.hwnd()), win_hotkey_id)
                .map_err(|_| HotkeyError::UnregistrationFailed)?;
        }

        self.registrations
            .lock()
            .unwrap()
            .remove(&win_hotkey_id);

        log::info!("Unregistered hotkey ID: {}", win_hotkey_id);
        Ok(())
    }

    /// Unregister all hotkeys for an app
    pub fn unregister_all_for_app(&self, app_id: &str) -> Result<(), HotkeyError> {
        let ids_to_remove: Vec<i32> = self
            .registrations
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, reg)| reg.app_id == app_id)
            .map(|(id, _)| *id)
            .collect();

        for id in ids_to_remove {
            self.unregister(id)?;
        }
        Ok(())
    }

    /// Check if a key combination is already registered
    pub fn is_hotkey_taken(&self, keys: &str) -> bool {
        self.registrations
            .lock()
            .unwrap()
            .values()
            .any(|reg| reg.keys.eq_ignore_ascii_case(keys))
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
    pub fn handle_hotkey_message(&self, wparam: usize) -> Option<HotkeyRegistration> {
        let hotkey_id = wparam as i32;
        self.registrations
            .lock()
            .unwrap()
            .get(&hotkey_id)
            .cloned()
    }

    /// Parse key string like "Alt+H" into Windows modifiers + virtual key
    fn parse_keys(keys: &str) -> Result<(HOT_KEY_MODIFIERS, u16), HotkeyError> {
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
                    vk_code = Some(Self::key_to_vk(key)?);
                }
            }
        }

        let vk = vk_code.ok_or(HotkeyError::InvalidKeyFormat)?;
        Ok((modifiers, vk))
    }

    /// Convert key name to Windows virtual key code
    fn key_to_vk(key: &str) -> Result<u16, HotkeyError> {
        let vk = match key.to_uppercase().as_str() {
            "A" => 0x41u16, "B" => 0x42, "C" => 0x43, "D" => 0x44,
            "E" => 0x45, "F" => 0x46, "G" => 0x47, "H" => 0x48,
            "I" => 0x49, "J" => 0x4A, "K" => 0x4B, "L" => 0x4C,
            "M" => 0x4D, "N" => 0x4E, "O" => 0x4F, "P" => 0x50,
            "Q" => 0x51, "R" => 0x52, "S" => 0x53, "T" => 0x54,
            "U" => 0x55, "V" => 0x56, "W" => 0x57, "X" => 0x58,
            "Y" => 0x59, "Z" => 0x5A,

            "0" => 0x30, "1" => 0x31, "2" => 0x32, "3" => 0x33,
            "4" => 0x34, "5" => 0x35, "6" => 0x36, "7" => 0x37,
            "8" => 0x38, "9" => 0x39,

            "F1" => VK_F1.0, "F2" => VK_F2.0,
            "F3" => VK_F3.0, "F4" => VK_F4.0,
            "F5" => VK_F5.0, "F6" => VK_F6.0,
            "F7" => VK_F7.0, "F8" => VK_F8.0,
            "F9" => VK_F9.0, "F10" => VK_F10.0,
            "F11" => VK_F11.0, "F12" => VK_F12.0,

            "SPACE" => VK_SPACE.0,
            "TAB" => VK_TAB.0,
            "ESC" | "ESCAPE" => VK_ESCAPE.0,
            "ENTER" | "RETURN" => VK_RETURN.0,

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
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )?;

            Ok(hwnd)
        }
    }

    /// Start message loop to listen for WM_HOTKEY messages
    pub fn start_message_loop(self: Arc<Self>, app_handle: tauri::AppHandle) {
        std::thread::spawn(move || unsafe {
            let mut msg = MSG::default();
            let hwnd = self.hwnd();

            loop {
                let result = GetMessageW(&mut msg, Some(hwnd), 0, 0);
                match result.0 {
                    -1 => break,
                    0 => break,
                    _ => {
                        if msg.message == WM_HOTKEY {
                            if let Some(registration) =
                                self.handle_hotkey_message(msg.wParam.0)
                            {
                                log::debug!(
                                    "Hotkey pressed: {} for app {}",
                                    registration.hotkey_id,
                                    registration.app_id
                                );

                                use tauri::Emitter;
                                let _ = app_handle.emit(
                                    &format!(
                                        "hotkey_pressed:{}",
                                        registration.app_id
                                    ),
                                    &registration,
                                );
                            }
                        }

                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }
        });
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        let registrations = self.registrations.lock().unwrap();
        for (id, _) in registrations.iter() {
            unsafe {
                let _ = UnregisterHotKey(Some(self.hwnd()), *id);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_hotkey() {
        let (modifiers, vk) = HotkeyManager::parse_keys("Alt+H").unwrap();
        assert_eq!(modifiers.0 & MOD_ALT.0, MOD_ALT.0);
        assert_eq!(vk, 0x48);
    }

    #[test]
    fn test_parse_multi_modifier_hotkey() {
        let (modifiers, vk) = HotkeyManager::parse_keys("Ctrl+Shift+F").unwrap();
        assert_eq!(modifiers.0 & MOD_CONTROL.0, MOD_CONTROL.0);
        assert_eq!(modifiers.0 & MOD_SHIFT.0, MOD_SHIFT.0);
        assert_eq!(vk, 0x46);
    }

    #[test]
    fn test_key_to_vk() {
        assert_eq!(HotkeyManager::key_to_vk("A").unwrap(), 0x41);
        assert_eq!(HotkeyManager::key_to_vk("Z").unwrap(), 0x5A);
        assert_eq!(HotkeyManager::key_to_vk("0").unwrap(), 0x30);
        assert_eq!(HotkeyManager::key_to_vk("F1").unwrap(), VK_F1.0);
    }

    #[test]
    fn test_invalid_key_format() {
        assert!(HotkeyManager::parse_keys("InvalidKey").is_err());
    }

    #[test]
    fn test_parse_keys_with_spaces() {
        let (modifiers, vk) = HotkeyManager::parse_keys("Ctrl + Shift + T").unwrap();
        assert_eq!(modifiers.0 & MOD_CONTROL.0, MOD_CONTROL.0);
        assert_eq!(modifiers.0 & MOD_SHIFT.0, MOD_SHIFT.0);
        assert_eq!(vk, 0x54);
    }
}
