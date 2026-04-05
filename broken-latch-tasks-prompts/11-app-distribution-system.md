# Task 11: App Distribution System

**Platform: broken-latch**  
**Dependencies:** Task 01, 07, 10  
**Estimated Complexity:** Medium  
**Priority:** P2 (Can be implemented after core platform works)

---

## Objective

Implement the app distribution and auto-update system that allows apps to be discovered, installed from a curated registry, and automatically updated. This includes building the app browser UI, implementing delta updates, and creating the update checker that runs on platform startup.

---

## Context

The distribution system provides:

1. **App Registry** - Curated JSON file hosted at `https://apps.broken-latch.gg/registry.json`
   - List of available apps with metadata
   - Download URLs for .lolapp packages
   - Screenshots, ratings, categories

2. **App Browser** - In-platform UI to browse and install apps
   - Featured apps section
   - Search and filter
   - One-click install

3. **Auto-Update System** - Checks for updates and applies them
   - Platform self-update on startup
   - Per-app update checking
   - Delta updates (only download changed files)
   - Background downloads

4. **Developer Publishing** - CLI tool for app developers
   - Package .lolapp from source
   - Publish to registry (future: developer portal)

---

## What You Need to Build

### 1. Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "stream"] }
tokio = { version = "1", features = ["fs"] }
sha2 = "0.10"                        # File hashing for delta updates
semver = "1.0"                       # Version comparison
```

### 2. App Registry Client (`src-tauri/src/apps/updater.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use reqwest;

const REGISTRY_URL: &str = "https://apps.broken-latch.gg/registry.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegistryEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub download_url: String,
    pub icon_url: String,
    pub screenshots: Vec<String>,
    pub min_platform_version: String,
    pub file_size: u64,
    pub checksum: String,  // SHA256
    pub rating: f32,
    pub download_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegistry {
    pub version: String,
    pub apps: Vec<AppRegistryEntry>,
}

pub struct RegistryClient {
    client: reqwest::Client,
}

impl RegistryClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch the app registry from remote server
    pub async fn fetch_registry(&self) -> Result<AppRegistry, UpdateError> {
        log::info!("Fetching app registry from {}", REGISTRY_URL);

        let response = self.client
            .get(REGISTRY_URL)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let registry: AppRegistry = response
            .json()
            .await
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        log::info!("Fetched {} apps from registry", registry.apps.len());

        Ok(registry)
    }

    /// Download an app from registry
    pub async fn download_app(
        &self,
        entry: &AppRegistryEntry,
        dest_path: PathBuf,
    ) -> Result<(), UpdateError> {
        log::info!("Downloading app {} from {}", entry.name, entry.download_url);

        let response = self.client
            .get(&entry.download_url)
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        // Verify checksum
        let hash = self.calculate_sha256(&bytes);
        if hash != entry.checksum {
            return Err(UpdateError::ChecksumMismatch);
        }

        // Write to file
        tokio::fs::write(&dest_path, bytes)
            .await
            .map_err(|e| UpdateError::IoError(e.to_string()))?;

        log::info!("Downloaded app to {:?}", dest_path);

        Ok(())
    }

    /// Check if an app has an update available
    pub async fn check_app_update(
        &self,
        app_id: &str,
        current_version: &str,
    ) -> Result<Option<AppRegistryEntry>, UpdateError> {
        let registry = self.fetch_registry().await?;

        let entry = registry.apps.iter()
            .find(|app| app.id == app_id)
            .ok_or(UpdateError::AppNotFound)?;

        // Compare versions
        let current = semver::Version::parse(current_version)
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;
        let latest = semver::Version::parse(&entry.version)
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        if latest > current {
            Ok(Some(entry.clone()))
        } else {
            Ok(None)
        }
    }

    fn calculate_sha256(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Checksum mismatch")]
    ChecksumMismatch,

    #[error("App not found in registry")]
    AppNotFound,
}
```

### 3. Platform Auto-Update (`src-tauri/src/apps/platform_updater.rs`)

```rust
use semver::Version;
use std::path::PathBuf;

const PLATFORM_UPDATE_URL: &str = "https://releases.broken-latch.gg/latest.json";

#[derive(Debug, Deserialize)]
struct PlatformRelease {
    version: String,
    download_url: String,
    checksum: String,
    release_notes: String,
}

pub struct PlatformUpdater {
    client: reqwest::Client,
}

impl PlatformUpdater {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Check for platform updates
    pub async fn check_for_update(&self) -> Result<Option<PlatformRelease>, UpdateError> {
        log::info!("Checking for platform updates...");

        let response = self.client
            .get(PLATFORM_UPDATE_URL)
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let release: PlatformRelease = response
            .json()
            .await
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        let current = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        let latest = Version::parse(&release.version)
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        if latest > current {
            log::info!("Platform update available: {} -> {}", current, latest);
            Ok(Some(release))
        } else {
            log::info!("Platform is up to date ({})", current);
            Ok(None)
        }
    }

    /// Download and apply platform update
    pub async fn download_update(
        &self,
        release: &PlatformRelease,
    ) -> Result<PathBuf, UpdateError> {
        log::info!("Downloading platform update v{}", release.version);

        let response = self.client
            .get(&release.download_url)
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        // Verify checksum
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = format!("{:x}", hasher.finalize());

        if hash != release.checksum {
            return Err(UpdateError::ChecksumMismatch);
        }

        // Save installer to temp directory
        let temp_path = std::env::temp_dir()
            .join(format!("broken-latch-update-{}.exe", release.version));

        tokio::fs::write(&temp_path, bytes)
            .await
            .map_err(|e| UpdateError::IoError(e.to_string()))?;

        log::info!("Update downloaded to {:?}", temp_path);

        Ok(temp_path)
    }

    /// Launch update installer and exit platform
    pub fn apply_update(&self, installer_path: PathBuf) -> Result<(), UpdateError> {
        log::info!("Launching update installer: {:?}", installer_path);

        // Launch installer
        std::process::Command::new(installer_path)
            .spawn()
            .map_err(|e| UpdateError::IoError(e.to_string()))?;

        // Exit current platform process
        std::process::exit(0);
    }
}

use serde::Deserialize;
```

### 4. App Browser UI Component (`src/components/AppBrowser/AppBrowser.tsx`)

```tsx
import React, { useState } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/tauri";
import { AppTile } from "./AppTile";
import { AppDetails } from "./AppDetails";
import { SearchBar } from "./SearchBar";
import { Download } from "lucide-react";

interface RegistryApp {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  category: string;
  download_url: string;
  icon_url: string;
  screenshots: string[];
  file_size: number;
  rating: number;
  download_count: number;
}

export function AppBrowser() {
  const [selectedApp, setSelectedApp] = useState<RegistryApp | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [categoryFilter, setCategoryFilter] = useState<string>("all");

  // Fetch app registry
  const { data: registry, isLoading } = useQuery({
    queryKey: ["app-registry"],
    queryFn: async () => {
      return await invoke<{ apps: RegistryApp[] }>("fetch_app_registry");
    },
  });

  // Install app mutation
  const installMutation = useMutation({
    mutationFn: async (app: RegistryApp) => {
      // Download app to temp, then install
      const tempPath = await invoke<string>("download_registry_app", {
        appId: app.id,
      });
      return await invoke("install_app", { lolappPath: tempPath });
    },
  });

  const filteredApps =
    registry?.apps.filter((app) => {
      const matchesSearch =
        app.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        app.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesCategory =
        categoryFilter === "all" || app.category === categoryFilter;
      return matchesSearch && matchesCategory;
    }) || [];

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-2">App Browser</h1>
          <p className="text-gray-400">
            Discover and install apps for broken-latch
          </p>
        </div>

        {/* Search and Filters */}
        <div className="mb-6 flex gap-4">
          <SearchBar
            value={searchQuery}
            onChange={setSearchQuery}
            placeholder="Search apps..."
          />

          <select
            value={categoryFilter}
            onChange={(e) => setCategoryFilter(e.target.value)}
            className="px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg"
          >
            <option value="all">All Categories</option>
            <option value="coaching">Coaching</option>
            <option value="stats">Stats & Analytics</option>
            <option value="utility">Utility</option>
            <option value="entertainment">Entertainment</option>
          </select>
        </div>

        {/* App Grid */}
        {isLoading ? (
          <div className="text-center py-12 text-gray-400">Loading apps...</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
            {filteredApps.map((app) => (
              <AppTile
                key={app.id}
                app={app}
                onClick={() => setSelectedApp(app)}
                onInstall={() => installMutation.mutate(app)}
                isInstalling={installMutation.isPending}
              />
            ))}
          </div>
        )}

        {filteredApps.length === 0 && !isLoading && (
          <div className="text-center py-12 text-gray-400">
            No apps found matching your criteria
          </div>
        )}
      </div>

      {/* App Details Modal */}
      {selectedApp && (
        <AppDetails
          app={selectedApp}
          onClose={() => setSelectedApp(null)}
          onInstall={() => installMutation.mutate(selectedApp)}
        />
      )}
    </div>
  );
}
```

### 5. Tauri Commands

```rust
#[tauri::command]
async fn fetch_app_registry(
    registry_client: tauri::State<'_, Arc<RegistryClient>>,
) -> Result<AppRegistry, String> {
    registry_client.fetch_registry().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_registry_app(
    app_id: String,
    registry_client: tauri::State<'_, Arc<RegistryClient>>,
) -> Result<String, String> {
    let registry = registry_client.fetch_registry().await
        .map_err(|e| e.to_string())?;

    let entry = registry.apps.iter()
        .find(|app| app.id == app_id)
        .ok_or("App not found in registry")?;

    let temp_path = std::env::temp_dir()
        .join(format!("{}.lolapp", app_id));

    registry_client.download_app(entry, temp_path.clone()).await
        .map_err(|e| e.to_string())?;

    Ok(temp_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn check_platform_update(
    platform_updater: tauri::State<'_, Arc<PlatformUpdater>>,
) -> Result<Option<PlatformRelease>, String> {
    platform_updater.check_for_update().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_platform_update(
    platform_updater: tauri::State<'_, Arc<PlatformUpdater>>,
    release: PlatformRelease,
) -> Result<String, String> {
    let path = platform_updater.download_update(&release).await
        .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn apply_platform_update(
    platform_updater: tauri::State<'_, Arc<PlatformUpdater>>,
    installer_path: String,
) -> Result<(), String> {
    platform_updater.apply_update(PathBuf::from(installer_path))
        .map_err(|e| e.to_string())
}
```

### 6. Update Checker on Startup (`src-tauri/src/main.rs`)

```rust
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();

            // Check for platform updates on startup (background task)
            let platform_updater = Arc::new(PlatformUpdater::new());
            let updater_clone = platform_updater.clone();

            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                match updater_clone.check_for_update().await {
                    Ok(Some(release)) => {
                        log::info!("Platform update available: v{}", release.version);
                        // Emit event to UI to show update notification
                        app_handle.emit_all("platform_update_available", &release).ok();
                    }
                    Ok(None) => {
                        log::info!("Platform is up to date");
                    }
                    Err(e) => {
                        log::warn!("Failed to check for updates: {}", e);
                    }
                }
            });

            app.manage(platform_updater);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Integration Points

### Uses Task 07 (App Lifecycle):

- Installs apps from registry
- Manages app updates

### Uses Task 10 (Platform UI):

- App Browser UI component
- Update notifications

---

## Testing Requirements

### Manual Testing Checklist

- [ ] Fetch app registry succeeds
- [ ] App browser displays apps correctly
- [ ] Search and filter work
- [ ] Download app from registry works
- [ ] Install downloaded app succeeds
- [ ] Checksum verification works
- [ ] Platform update check works on startup
- [ ] Platform update download works
- [ ] Update installer launches correctly

---

## Acceptance Criteria

✅ **Complete when:**

1. App registry can be fetched from remote URL
2. App browser UI displays all registry apps
3. Apps can be downloaded and installed from browser
4. Checksum verification prevents corrupted downloads
5. Platform update check runs on startup
6. Platform updates can be downloaded and applied
7. All manual tests pass
8. Update notifications appear in UI

---

## Files to Create/Modify

### New Files:

- `src-tauri/src/apps/updater.rs`
- `src-tauri/src/apps/platform_updater.rs`
- `src/components/AppBrowser/*`

### Modified Files:

- `src-tauri/src/main.rs`
- `src-tauri/Cargo.toml`

---

## Expected Time: 10-12 hours

## Difficulty: Medium (HTTP downloads + version management + UI)
