use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub async fn execute(path: &str) -> Result<()> {
    let project_dir = PathBuf::from(path);
    let mut errors: Vec<String> = Vec::new();

    println!("{} Validating app in '{}'...", "→".blue(), path);

    let manifest_path = project_dir.join("manifest.json");
    if !manifest_path.exists() {
        errors.push("manifest.json not found".to_string());
    } else {
        match fs::read_to_string(&manifest_path) {
            Err(e) => errors.push(format!("Could not read manifest.json: {}", e)),
            Ok(content) => {
                match serde_json::from_str::<crate::utils::manifest::AppManifest>(&content) {
                    Err(e) => errors.push(format!("Invalid JSON in manifest.json: {}", e)),
                    Ok(manifest) => {
                        // Field-level validation
                        errors.extend(manifest.validate());

                        // Entry point exists
                        let entry = project_dir.join(&manifest.entry_point);
                        if !entry.exists() {
                            errors.push(format!(
                                "entry_point '{}' not found",
                                manifest.entry_point
                            ));
                        }

                        // Window URLs exist (skip http:// URLs used in dev)
                        for window in &manifest.windows {
                            if !window.url.starts_with("http") {
                                let wp = project_dir.join(&window.url);
                                if !wp.exists() {
                                    errors.push(format!(
                                        "window '{}' url '{}' not found",
                                        window.id, window.url
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        println!("{} Validation passed", "✓".green());
        Ok(())
    } else {
        println!("\n{} Validation failed ({} error(s)):\n", "✗".red(), errors.len());
        for e in &errors {
            println!("  {} {}", "•".red(), e);
        }
        anyhow::bail!("Validation failed with {} error(s)", errors.len())
    }
}
