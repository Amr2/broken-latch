use anyhow::Result;
use colored::Colorize;
use notify::{Event, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

pub async fn execute(path: &str) -> Result<()> {
    let project_dir = PathBuf::from(path).canonicalize()?;

    println!("{} Starting development watcher...", "→".blue());
    println!("Watching: {}", project_dir.display());
    println!("Press Ctrl+C to stop\n");

    // Validate manifest first
    crate::commands::validate::execute(path).await?;

    let (tx, rx) = channel::<Result<Event, notify::Error>>();

    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&project_dir, RecursiveMode::Recursive)?;

    println!("{} Watching for changes…\n", "✓".green());

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(event)) => {
                if should_reload(&event) {
                    let changed: Vec<String> = event
                        .paths
                        .iter()
                        .filter_map(|p| p.file_name()?.to_str().map(str::to_owned))
                        .collect();
                    println!("{} Changed: {}", "↻".yellow(), changed.join(", "));

                    // Notify the running platform via HTTP API
                    reload_platform_app(&project_dir).await;

                    println!("{} Reload signal sent\n", "✓".green());
                }
            }
            Ok(Err(e)) => eprintln!("Watch error: {}", e),
            Err(_) => {} // timeout — keep watching
        }
    }
}

fn should_reload(event: &Event) -> bool {
    use notify::EventKind;
    matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
        && event.paths.iter().any(|p| {
            matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("html") | Some("js") | Some("css") | Some("json")
            )
        })
}

async fn reload_platform_app(_project_dir: &PathBuf) {
    // Signal platform to reload webviews — best-effort
    let _ = reqwest::Client::new()
        .post("http://localhost:45678/dev/reload")
        .send()
        .await;
}
