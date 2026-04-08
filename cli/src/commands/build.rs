use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub async fn execute(path: &str, output: &str) -> Result<()> {
    let project_dir = PathBuf::from(path);
    let output_dir = project_dir.join(output);

    println!("{} Building app to '{}'...", "→".blue(), output);

    // Validate first
    crate::commands::validate::execute(path).await?;

    // Create output directory
    fs::create_dir_all(&output_dir)?;

    // Copy all non-excluded files to dist
    copy_dir(&project_dir, &output_dir, &project_dir)?;

    println!("{} Build complete: {}", "✓".green(), output_dir.display());
    Ok(())
}

fn copy_dir(
    src: &std::path::Path,
    dst: &std::path::Path,
    base: &std::path::Path,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip excluded dirs and the output dir itself
        if matches!(
            name,
            "node_modules" | ".git" | ".vscode" | ".DS_Store" | "Thumbs.db" | ".env"
        ) || name.ends_with(".lolapp")
        {
            continue;
        }

        let rel = path.strip_prefix(base)?;
        let dest = dst.join(rel);

        if path.is_dir() {
            fs::create_dir_all(&dest)?;
            copy_dir(&path, dst, base)?;
        } else if path.is_file() {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}
