# Task 10: Platform UI

**Platform: broken-latch**  
**Dependencies:** Task 01, 07  
**Estimated Complexity:** Medium  
**Priority:** P1 (Important but not blocking apps)

---

## Objective

Build the platform's user-facing UI including the system tray icon/menu, app manager window, platform settings window, and app browser. This is how users interact with broken-latch to install/manage apps and configure platform settings.

---

## Context

The platform UI consists of:

1. **System Tray** - Always visible in Windows taskbar tray
   - Shows platform status (game detected, apps running)
   - Quick access menu (settings, app manager, quit)

2. **App Manager Window** - Main control panel
   - List of installed apps with enable/disable toggles
   - Install new app button
   - Uninstall apps
   - Per-app settings access

3. **Platform Settings Window**
   - Startup behavior (launch with Windows)
   - Overlay opacity defaults
   - Performance mode toggle
   - Developer mode settings

4. **App Browser** - Browse and install apps from registry
   - Featured apps
   - Search and filter
   - One-click install

---

## What You Need to Build

### 1. Dependencies

Frontend packages (`package.json`):

```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "zustand": "^4.4.0",
    "@tanstack/react-query": "^5.0.0",
    "lucide-react": "^0.300.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "@vitejs/plugin-react": "^4.2.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  }
}
```

### 2. System Tray (`src-tauri/src/tray.rs`)

```rust
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

pub fn create_system_tray() -> SystemTray {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("status".to_string(), "Platform Ready"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("apps".to_string(), "Manage Apps"))
        .add_item(CustomMenuItem::new("settings".to_string(), "Settings"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("toggle_overlays".to_string(), "Disable All Overlays"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit broken-latch"));

    SystemTray::new().with_menu(tray_menu)
}

pub fn handle_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "apps" => {
                open_app_manager(app);
            }
            "settings" => {
                open_settings(app);
            }
            "toggle_overlays" => {
                toggle_all_overlays(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        },
        _ => {}
    }
}

fn open_app_manager(app: &AppHandle) {
    if let Some(window) = app.get_window("app-manager") {
        window.show().ok();
        window.set_focus().ok();
    } else {
        tauri::WindowBuilder::new(
            app,
            "app-manager",
            tauri::WindowUrl::App("index.html#/apps".into()),
        )
        .title("broken-latch - App Manager")
        .inner_size(800.0, 600.0)
        .resizable(true)
        .center()
        .build()
        .ok();
    }
}

fn open_settings(app: &AppHandle) {
    if let Some(window) = app.get_window("settings") {
        window.show().ok();
        window.set_focus().ok();
    } else {
        tauri::WindowBuilder::new(
            app,
            "settings",
            tauri::WindowUrl::App("index.html#/settings".into()),
        )
        .title("broken-latch - Settings")
        .inner_size(600.0, 500.0)
        .resizable(false)
        .center()
        .build()
        .ok();
    }
}

fn toggle_all_overlays(app: &AppHandle) {
    // TODO: Implement overlay toggle
    log::info!("Toggle all overlays");
}
```

### 3. Frontend Structure

```
src/
├── components/
│   ├── AppManager/
│   │   ├── AppManager.tsx
│   │   ├── AppCard.tsx
│   │   ├── InstallAppDialog.tsx
│   │   └── AppSettings.tsx
│   ├── Settings/
│   │   ├── Settings.tsx
│   │   ├── GeneralSettings.tsx
│   │   ├── OverlaySettings.tsx
│   │   ├── PerformanceSettings.tsx
│   │   └── DeveloperSettings.tsx
│   ├── AppBrowser/
│   │   ├── AppBrowser.tsx
│   │   ├── AppTile.tsx
│   │   ├── AppDetails.tsx
│   │   └── SearchBar.tsx
│   └── shared/
│       ├── Button.tsx
│       ├── Switch.tsx
│       ├── Dialog.tsx
│       └── Toast.tsx
├── hooks/
│   ├── useInstalledApps.ts
│   ├── usePlatformConfig.ts
│   └── useAppRegistry.ts
├── lib/
│   ├── api.ts
│   └── types.ts
├── App.tsx
├── main.tsx
└── index.css
```

### 4. App Manager Component (`src/components/AppManager/AppManager.tsx`)

```tsx
import React, { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { AppCard } from "./AppCard";
import { InstallAppDialog } from "./InstallAppDialog";
import { Plus, RefreshCw } from "lucide-react";

interface InstalledApp {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  state: string;
}

export function AppManager() {
  const [showInstallDialog, setShowInstallDialog] = useState(false);
  const queryClient = useQueryClient();

  // Fetch installed apps
  const { data: apps, isLoading } = useQuery({
    queryKey: ["installed-apps"],
    queryFn: async () => {
      return await invoke<InstalledApp[]>("list_installed_apps");
    },
  });

  // Install app mutation
  const installMutation = useMutation({
    mutationFn: async (lolappPath: string) => {
      return await invoke("install_app", { lolappPath });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] });
    },
  });

  // Start/stop app mutations
  const startAppMutation = useMutation({
    mutationFn: async (appId: string) => {
      return await invoke("start_app", { appId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] });
    },
  });

  const stopAppMutation = useMutation({
    mutationFn: async (appId: string) => {
      return await invoke("stop_app", { appId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] });
    },
  });

  // Uninstall app mutation
  const uninstallMutation = useMutation({
    mutationFn: async (appId: string) => {
      return await invoke("uninstall_app", { appId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] });
    },
  });

  const handleInstallClick = async () => {
    const selected = await open({
      filters: [{ name: "broken-latch App", extensions: ["lolapp"] }],
    });

    if (selected && typeof selected === "string") {
      installMutation.mutate(selected);
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold">App Manager</h1>
            <p className="text-gray-400 mt-1">
              Manage your installed broken-latch apps
            </p>
          </div>
          <div className="flex gap-3">
            <button
              onClick={() =>
                queryClient.invalidateQueries({ queryKey: ["installed-apps"] })
              }
              className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg flex items-center gap-2"
            >
              <RefreshCw size={18} />
              Refresh
            </button>
            <button
              onClick={handleInstallClick}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-lg flex items-center gap-2"
            >
              <Plus size={18} />
              Install App
            </button>
          </div>
        </div>

        {/* App Grid */}
        {isLoading ? (
          <div className="text-center py-12 text-gray-400">Loading apps...</div>
        ) : apps && apps.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {apps.map((app) => (
              <AppCard
                key={app.id}
                app={app}
                onStart={() => startAppMutation.mutate(app.id)}
                onStop={() => stopAppMutation.mutate(app.id)}
                onUninstall={() => {
                  if (confirm(`Uninstall ${app.name}?`)) {
                    uninstallMutation.mutate(app.id);
                  }
                }}
              />
            ))}
          </div>
        ) : (
          <div className="text-center py-12">
            <p className="text-gray-400 mb-4">No apps installed yet</p>
            <button
              onClick={handleInstallClick}
              className="px-6 py-3 bg-blue-600 hover:bg-blue-500 rounded-lg"
            >
              Install Your First App
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
```

### 5. App Card Component (`src/components/AppManager/AppCard.tsx`)

```tsx
import React from "react";
import { Play, Square, Trash2, Settings } from "lucide-react";

interface AppCardProps {
  app: {
    id: string;
    name: string;
    version: string;
    description: string;
    author: string;
    enabled: boolean;
    state: string;
  };
  onStart: () => void;
  onStop: () => void;
  onUninstall: () => void;
}

export function AppCard({ app, onStart, onStop, onUninstall }: AppCardProps) {
  const isRunning = app.state === "Running";

  return (
    <div className="bg-gray-800 rounded-lg p-6 border border-gray-700 hover:border-gray-600 transition">
      {/* App Info */}
      <div className="mb-4">
        <h3 className="text-xl font-semibold mb-1">{app.name}</h3>
        <p className="text-sm text-gray-400 mb-2">
          v{app.version} by {app.author}
        </p>
        <p className="text-gray-300 text-sm">{app.description}</p>
      </div>

      {/* State Badge */}
      <div className="mb-4">
        <span
          className={`inline-block px-3 py-1 rounded-full text-xs font-medium ${
            isRunning
              ? "bg-green-900 text-green-300"
              : "bg-gray-700 text-gray-300"
          }`}
        >
          {app.state}
        </span>
      </div>

      {/* Actions */}
      <div className="flex gap-2">
        {isRunning ? (
          <button
            onClick={onStop}
            className="flex-1 px-4 py-2 bg-red-600 hover:bg-red-500 rounded-lg flex items-center justify-center gap-2"
          >
            <Square size={16} />
            Stop
          </button>
        ) : (
          <button
            onClick={onStart}
            className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-500 rounded-lg flex items-center justify-center gap-2"
          >
            <Play size={16} />
            Start
          </button>
        )}

        <button
          className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg"
          title="App Settings"
        >
          <Settings size={16} />
        </button>

        <button
          onClick={onUninstall}
          className="px-4 py-2 bg-gray-700 hover:bg-red-600 rounded-lg"
          title="Uninstall"
        >
          <Trash2 size={16} />
        </button>
      </div>
    </div>
  );
}
```

### 6. Platform Settings Component (`src/components/Settings/Settings.tsx`)

```tsx
import React from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/tauri";
import { GeneralSettings } from "./GeneralSettings";
import { OverlaySettings } from "./OverlaySettings";
import { PerformanceSettings } from "./PerformanceSettings";
import { DeveloperSettings } from "./DeveloperSettings";

interface PlatformConfig {
  startup: {
    launch_with_windows: boolean;
    auto_update_check: boolean;
  };
  overlay: {
    default_opacity: number;
    screen_capture_visible: boolean;
  };
  performance: {
    performance_mode: boolean;
    cpu_usage_limit: number;
  };
  developer: {
    debug_mode: boolean;
    show_console_logs: boolean;
    simulated_game_phases: boolean;
  };
}

export function Settings() {
  const { data: config, isLoading } = useQuery({
    queryKey: ["platform-config"],
    queryFn: async () => {
      return await invoke<PlatformConfig>("get_platform_config");
    },
  });

  const updateConfigMutation = useMutation({
    mutationFn: async (newConfig: Partial<PlatformConfig>) => {
      return await invoke("update_platform_config", { config: newConfig });
    },
  });

  if (isLoading || !config) {
    return <div className="p-8 text-gray-400">Loading settings...</div>;
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-3xl font-bold mb-8">Platform Settings</h1>

        <div className="space-y-8">
          <GeneralSettings
            config={config.startup}
            onChange={(startup) => updateConfigMutation.mutate({ startup })}
          />

          <OverlaySettings
            config={config.overlay}
            onChange={(overlay) => updateConfigMutation.mutate({ overlay })}
          />

          <PerformanceSettings
            config={config.performance}
            onChange={(performance) =>
              updateConfigMutation.mutate({ performance })
            }
          />

          <DeveloperSettings
            config={config.developer}
            onChange={(developer) => updateConfigMutation.mutate({ developer })}
          />
        </div>
      </div>
    </div>
  );
}
```

### 7. Tauri Commands for UI

```rust
#[tauri::command]
async fn get_platform_config(
    config: tauri::State<'_, Arc<Mutex<PlatformConfig>>>,
) -> Result<PlatformConfig, String> {
    Ok(config.lock().unwrap().clone())
}

#[tauri::command]
async fn update_platform_config(
    config_state: tauri::State<'_, Arc<Mutex<PlatformConfig>>>,
    config: PlatformConfig,
) -> Result<(), String> {
    *config_state.lock().unwrap() = config.clone();
    crate::config::save_config(&config)
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

---

## Integration Points

### Uses Task 07 (App Lifecycle):

- Lists installed apps
- Installs/uninstalls apps
- Starts/stops apps

### Uses Task 01 (Config):

- Loads and saves platform configuration

---

## Testing Requirements

### Manual Testing Checklist

- [ ] System tray icon appears
- [ ] Tray menu opens correctly
- [ ] App Manager window opens from tray
- [ ] Installed apps list displays
- [ ] Install app dialog works
- [ ] Start/stop app buttons work
- [ ] Uninstall app works
- [ ] Settings window opens
- [ ] Settings persist across restarts
- [ ] UI is responsive and styled correctly

---

## Acceptance Criteria

✅ **Complete when:**

1. System tray is fully functional
2. App Manager displays all installed apps
3. Apps can be installed via file picker
4. Apps can be started and stopped from UI
5. Apps can be uninstalled
6. Settings window allows configuration
7. All settings persist correctly
8. UI is polished and user-friendly
9. Manual testing checklist is 100% complete

---

## Files to Create/Modify

### New Files:

- `src-tauri/src/tray.rs`
- `src/components/AppManager/*`
- `src/components/Settings/*`
- Various UI components

### Modified Files:

- `src-tauri/src/main.rs`
- `src/App.tsx`

---

## Expected Time: 12-14 hours

## Difficulty: Medium (UI development + Tauri integration)
