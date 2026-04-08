use anyhow::{Context, Result};
use colored::Colorize;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub async fn execute(path: &str, output: Option<String>) -> Result<()> {
    let project_dir = PathBuf::from(path);

    println!("{} Packaging app...", "→".blue());

    // Validate first
    crate::commands::validate::execute(path).await?;

    // Read manifest
    let manifest_path = project_dir.join("manifest.json");
    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;

    let app_id = manifest["id"]
        .as_str()
        .context("manifest.json missing 'id' field")?;
    let version = manifest["version"]
        .as_str()
        .context("manifest.json missing 'version' field")?;

    let output_path = output.unwrap_or_else(|| format!("{}-{}.lolapp", app_id, version));

    let file = File::create(&output_path)
        .with_context(|| format!("Failed to create output file '{}'", output_path))?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut zip, &project_dir, &project_dir, options)?;
    zip.finish()?;

    println!("{} Package created: {}", "✓".green(), output_path.bold());
    Ok(())
}

fn add_dir_to_zip(
    zip: &mut ZipWriter<File>,
    base_dir: &Path,
    current_dir: &Path,
    options: SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(current_dir).context("Failed to read directory")? {
        let entry = entry?;
        let path = entry.path();

        if should_exclude(&path) {
            continue;
        }

        let rel = path.strip_prefix(base_dir)?;
        let name = rel.to_string_lossy().replace('\\', "/");

        if path.is_file() {
            zip.start_file(&name, options)?;
            let mut f = File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        } else if path.is_dir() {
            add_dir_to_zip(zip, base_dir, &path, options)?;
        }
    }
    Ok(())
}

fn should_exclude(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    matches!(
        name,
        "node_modules" | ".git" | ".vscode" | "dist" | "build" | ".DS_Store" | "Thumbs.db" | ".env"
    ) || name.ends_with(".lolapp")
}
