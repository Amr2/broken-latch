use anyhow::Result;
use colored::Colorize;

pub async fn execute(path: &str) -> Result<()> {
    println!("{} Publishing is not yet implemented.", "!".yellow());
    println!("Package your app first with:");
    println!("  blatch package --path {}", path);
    println!("\nThen submit it at https://broken-latch.gg/developers");
    Ok(())
}
