# Task 13: Developer CLI

**Platform: broken-latch**  
**Dependencies:** Task 01, 07, 09  
**Estimated Complexity:** Medium  
**Priority:** P2 (Quality of life for developers)

---

## Objective

Build a command-line interface (CLI) tool for app developers to streamline the development workflow. The CLI should handle project scaffolding, local development with hot reload, packaging apps into `.lolapp` files, and (future) publishing to the app registry.

---

## Context

The Developer CLI (`loloverlay-cli` or `blatch`) provides:

1. **Project Scaffolding** - Generate new app from template
2. **Local Development** - Hot reload during development
3. **Build & Package** - Create `.lolapp` distribution files
4. **Validation** - Lint manifest and check for common errors
5. **Testing** - Simulate game phases without running League
6. **Publishing** - (Future) Publish to app registry

Developer workflow:

```bash
# Create new app
blatch new my-overlay-app

# Start dev mode with hot reload
blatch dev

# Build for distribution
blatch build

# Package as .lolapp
blatch package

# Publish to registry (future)
blatch publish
```

---

## What You Need to Build

### 1. CLI Project Structure

```
cli/
├── src/
│   ├── main.rs                   # Entry point, command parsing
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── new.rs                # Create new project
│   │   ├── dev.rs                # Development mode
│   │   ├── build.rs              # Build app
│   │   ├── package.rs            # Package as .lolapp
│   │   ├── validate.rs           # Validate manifest
│   │   └── publish.rs            # Publish to registry
│   ├── templates/
│   │   ├── mod.rs
│   │   ├── basic.rs              # Basic app template
│   │   └── react.rs              # React + TypeScript template
│   └── utils/
│       ├── mod.rs
│       ├── manifest.rs           # Manifest parsing/validation
│       └── watcher.rs            # File watcher for hot reload
├── Cargo.toml
└── README.md
```

### 2. Dependencies (`cli/Cargo.toml`)

```toml
[package]
name = "broken-latch-cli"
version = "1.0.0"
edition = "2021"

[[bin]]
name = "blatch"
path = "src/main.rs"

[dependencies]
clap = { version = "4.4", features = ["derive", "cargo"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
colored = "2.1"
dialoguer = "0.11"
indicatif = "0.17"
notify = "6.1"                    # File system watcher
zip = "0.6"
reqwest = { version = "0.11", features = ["blocking", "json"] }
validator = { version = "0.16", features = ["derive"] }
```

### 3. Main Entry Point (`cli/src/main.rs`)

```rust
use clap::{Parser, Subcommand};
use colored::*;

mod commands;
mod templates;
mod utils;

#[derive(Parser)]
#[command(name = "blatch")]
#[command(about = "broken-latch developer CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new broken-latch app from template
    New {
        /// Name of the app
        name: String,

        /// Template to use (basic, react)
        #[arg(short, long, default_value = "basic")]
        template: String,
    },

    /// Start development server with hot reload
    Dev {
        /// Path to app directory
        #[arg(default_value = ".")]
        path: String,
    },

    /// Build app for distribution
    Build {
        /// Path to app directory
        #[arg(default_value = ".")]
        path: String,

        /// Output directory
        #[arg(short, long, default_value = "dist")]
        output: String,
    },

    /// Package app as .lolapp file
    Package {
        /// Path to app directory
        #[arg(default_value = ".")]
        path: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Validate app manifest and structure
    Validate {
        /// Path to app directory
        #[arg(default_value = ".")]
        path: String,
    },

    /// Publish app to registry (requires authentication)
    Publish {
        /// Path to app directory
        #[arg(default_value = ".")]
        path: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, template } => {
            commands::new::execute(&name, &template).await?;
        }
        Commands::Dev { path } => {
            commands::dev::execute(&path).await?;
        }
        Commands::Build { path, output } => {
            commands::build::execute(&path, &output).await?;
        }
        Commands::Package { path, output } => {
            commands::package::execute(&path, output).await?;
        }
        Commands::Validate { path } => {
            commands::validate::execute(&path).await?;
        }
        Commands::Publish { path } => {
            commands::publish::execute(&path).await?;
        }
    }

    Ok(())
}
```

### 4. New Project Command (`cli/src/commands/new.rs`)

````rust
use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;

pub async fn execute(name: &str, template: &str) -> Result<()> {
    println!("{} Creating new broken-latch app: {}", "✓".green(), name.bold());

    // Create project directory
    let project_dir = PathBuf::from(name);
    if project_dir.exists() {
        anyhow::bail!("Directory {} already exists", name);
    }

    fs::create_dir_all(&project_dir)
        .context("Failed to create project directory")?;

    // Generate files based on template
    match template {
        "basic" => generate_basic_template(&project_dir, name)?,
        "react" => generate_react_template(&project_dir, name)?,
        _ => anyhow::bail!("Unknown template: {}", template),
    }

    println!("\n{} Project created successfully!", "✓".green());
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  blatch dev");

    Ok(())
}

fn generate_basic_template(dir: &PathBuf, name: &str) -> Result<()> {
    // Create manifest.json
    let manifest = serde_json::json!({
        "id": name,
        "name": name,
        "version": "1.0.0",
        "description": "A broken-latch overlay app",
        "author": "Your Name",
        "entryPoint": "index.html",
        "permissions": ["game.session", "windows.create"],
        "windows": [{
            "id": "main",
            "url": "index.html",
            "defaultPosition": { "x": 20, "y": 100 },
            "defaultSize": { "width": 320, "height": 480 },
            "draggable": true,
            "resizable": false,
            "persistPosition": true,
            "opacity": 0.9,
            "showInPhases": ["InGame"]
        }]
    });

    fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    // Create index.html
    let html = r#"<!DOCTYPE html>
<html>
<head>
  <title>My Overlay App</title>
  <style>
    body {
      margin: 0;
      padding: 20px;
      background: rgba(0, 0, 0, 0.85);
      color: white;
      font-family: 'Segoe UI', Tahoma, sans-serif;
    }

    .header {
      font-size: 18px;
      font-weight: bold;
      margin-bottom: 15px;
      cursor: move;
    }

    .content {
      font-size: 14px;
    }
  </style>
</head>
<body>
  <div class="header" data-broken-latch-drag="true">
    My Overlay App
  </div>
  <div class="content" id="content">
    Waiting for game...
  </div>

  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script src="app.js"></script>
</body>
</html>
"#;

    fs::write(dir.join("index.html"), html)?;

    // Create app.js
    let js = r#"// Initialize SDK
LOLOverlay.init({
  appId: 'your-app-id',
  version: '1.0.0'
});

const contentEl = document.getElementById('content');

// Listen for game phase changes
LOLOverlay.game.onPhaseChange(async (event) => {
  console.log(`Phase changed: ${event.previous} -> ${event.current}`);

  if (event.current === 'InGame') {
    const session = await LOLOverlay.game.getSession();

    if (session) {
      contentEl.innerHTML = `
        <strong>Game started!</strong><br>
        Local player: ${session.localPlayer.summonerName}<br>
        Champion: ${session.localPlayer.championName}
      `;
    }
  } else {
    contentEl.textContent = 'Waiting for game...';
  }
});
"#;

    fs::write(dir.join("app.js"), js)?;

    // Create README.md
    let readme = format!(r#"# {}

A broken-latch overlay app.

## Development

```bash
blatch dev
````

## Build

```bash
blatch build
blatch package
```

"#, name);

    fs::write(dir.join("README.md"), readme)?;

    Ok(())

}

fn generate_react_template(dir: &PathBuf, name: &str) -> Result<()> {
// TODO: Implement React template with Vite + TypeScript
anyhow::bail!("React template not yet implemented");
}

````

### 5. Dev Mode Command (`cli/src/commands/dev.rs`)

```rust
use anyhow::{Context, Result};
use colored::*;
use notify::{Watcher, RecursiveMode, Event};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

pub async fn execute(path: &str) -> Result<()> {
    let project_dir = PathBuf::from(path);

    println!("{} Starting development server...", "→".blue());
    println!("Watching for file changes in: {}", project_dir.display());
    println!("Press Ctrl+C to stop\n");

    // Validate manifest first
    crate::commands::validate::execute(path).await?;

    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            tx.send(event).ok();
        }
    })?;

    watcher.watch(&project_dir, RecursiveMode::Recursive)?;

    println!("{} Watching for changes...\n", "✓".green());

    // Watch for file changes
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                if should_reload(&event) {
                    println!("{} File changed, reloading app...", "↻".yellow());

                    // Signal platform to reload app
                    reload_app_in_platform(&project_dir).await?;

                    println!("{} App reloaded\n", "✓".green());
                }
            }
            Err(_) => {
                // Timeout, continue watching
            }
        }
    }
}

fn should_reload(event: &Event) -> bool {
    // Only reload for modify events on .html, .js, .css files
    if let notify::EventKind::Modify(_) = event.kind {
        event.paths.iter().any(|p| {
            matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("html") | Some("js") | Some("css") | Some("json")
            )
        })
    } else {
        false
    }
}

async fn reload_app_in_platform(project_dir: &PathBuf) -> Result<()> {
    // TODO: Call platform API to reload app
    // For now, just validate
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(())
}
````

### 6. Package Command (`cli/src/commands/package.rs`)

```rust
use anyhow::{Context, Result};
use colored::*;
use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::{Path, PathBuf};
use zip::ZipWriter;
use zip::write::FileOptions;

pub async fn execute(path: &str, output: Option<String>) -> Result<()> {
    let project_dir = PathBuf::from(path);

    println!("{} Packaging app...", "→".blue());

    // Validate first
    crate::commands::validate::execute(path).await?;

    // Read manifest to get app ID and version
    let manifest_path = project_dir.join("manifest.json");
    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;

    let app_id = manifest["id"].as_str()
        .context("manifest.json missing 'id' field")?;
    let version = manifest["version"].as_str()
        .context("manifest.json missing 'version' field")?;

    // Determine output file path
    let output_path = output.unwrap_or_else(|| {
        format!("{}-{}.lolapp", app_id, version)
    });

    // Create .lolapp (zip) file
    let file = File::create(&output_path)?;
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Add all files to zip (excluding dev files)
    add_dir_to_zip(&mut zip, &project_dir, &project_dir, options)?;

    zip.finish()?;

    println!("{} Package created: {}", "✓".green(), output_path.bold());

    Ok(())
}

fn add_dir_to_zip(
    zip: &mut ZipWriter<File>,
    base_dir: &Path,
    current_dir: &Path,
    options: FileOptions,
) -> Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(base_dir)?;

        // Skip excluded files/directories
        if should_exclude(&path) {
            continue;
        }

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(&path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else if path.is_dir() {
            add_dir_to_zip(zip, base_dir, &path, options)?;
        }
    }

    Ok(())
}

fn should_exclude(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Exclude dev files and directories
    matches!(name,
        "node_modules" | ".git" | ".vscode" | "dist" | "build" |
        ".DS_Store" | "Thumbs.db" | ".env"
    )
}
```

### 7. Validate Command (`cli/src/commands/validate.rs`)

```rust
use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;
use validator::Validate;

pub async fn execute(path: &str) -> Result<()> {
    let project_dir = PathBuf::from(path);
    let mut errors = Vec::new();

    println!("{} Validating app...", "→".blue());

    // Check manifest.json exists
    let manifest_path = project_dir.join("manifest.json");
    if !manifest_path.exists() {
        errors.push("manifest.json not found".to_string());
    } else {
        // Parse and validate manifest
        let content = fs::read_to_string(&manifest_path)?;
        match serde_json::from_str::<crate::utils::manifest::AppManifest>(&content) {
            Ok(manifest) => {
                // Validate with validator crate
                if let Err(e) = manifest.validate() {
                    errors.push(format!("Manifest validation failed: {}", e));
                }

                // Check entry point exists
                let entry_point = project_dir.join(&manifest.entry_point);
                if !entry_point.exists() {
                    errors.push(format!("Entry point '{}' not found", manifest.entry_point));
                }

                // Check window URLs exist
                for window in &manifest.windows {
                    let window_path = project_dir.join(&window.url);
                    if !window_path.exists() {
                        errors.push(format!("Window URL '{}' not found", window.url));
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Invalid manifest.json: {}", e));
            }
        }
    }

    // Report results
    if errors.is_empty() {
        println!("{} Validation passed", "✓".green());
        Ok(())
    } else {
        println!("\n{} Validation failed with {} error(s):\n", "✗".red(), errors.len());
        for error in errors {
            println!("  {} {}", "•".red(), error);
        }
        anyhow::bail!("Validation failed");
    }
}
```

---

## Installation

The CLI will be distributed as:

1. **Cargo Install:**

   ```bash
   cargo install broken-latch-cli
   ```

2. **NPM (wrapper):**

   ```bash
   npm install -g @broken-latch/cli
   ```

3. **Binary Download:**
   - Windows: `blatch.exe`
   - Pre-built binaries on GitHub Releases

---

## Testing Requirements

### Manual Testing Checklist

- [ ] `blatch new my-app` creates project structure
- [ ] `blatch dev` starts file watcher
- [ ] File changes trigger reload
- [ ] `blatch validate` catches manifest errors
- [ ] `blatch build` compiles app
- [ ] `blatch package` creates valid .lolapp file
- [ ] Generated .lolapp can be installed in platform
- [ ] Help text displays correctly
- [ ] Error messages are clear

---

## Acceptance Criteria

✅ **Complete when:**

1. CLI compiles and runs on Windows
2. `new` command generates working app template
3. `dev` command watches for file changes
4. `validate` command checks manifest and structure
5. `package` command creates valid .lolapp files
6. All commands have proper error handling
7. Help text is complete and accurate
8. Manual testing checklist is 100% complete
9. CLI is published to crates.io and npm

---

## Files to Create

### New Files:

- `cli/src/*` - All CLI source files
- `cli/Cargo.toml`
- `cli/README.md`

---

## Expected Time: 10-12 hours

## Difficulty: Medium (CLI development + file operations + validation)
