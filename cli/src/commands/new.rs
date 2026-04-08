use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub async fn execute(name: &str, template: &str) -> Result<()> {
    println!("{} Creating new broken-latch app: {}", "✓".green(), name.bold());

    let project_dir = PathBuf::from(name);
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(&project_dir).context("Failed to create project directory")?;

    match template {
        "basic" => generate_basic_template(&project_dir, name)?,
        "react" => anyhow::bail!("React template not yet implemented — use 'basic'"),
        _ => anyhow::bail!("Unknown template: '{}'. Available: basic", template),
    }

    println!("\n{} Project '{}' created!", "✓".green(), name.bold());
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  blatch dev");

    Ok(())
}

fn generate_basic_template(dir: &PathBuf, name: &str) -> Result<()> {
    let manifest = serde_json::json!({
        "id": name,
        "name": name,
        "version": "1.0.0",
        "description": "A broken-latch overlay app",
        "author": "Your Name",
        "entry_point": "index.html",
        "permissions": ["game.session", "windows.create"],
        "windows": [{
            "id": "main",
            "url": "index.html",
            "default_position": { "x": 20, "y": 100 },
            "default_size": { "width": 320, "height": 480 },
            "draggable": true,
            "resizable": false,
            "persist_position": true,
            "opacity": 0.9,
            "show_in_phases": ["InGame"]
        }]
    });

    fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>{name}</title>
  <style>
    body {{
      margin: 0;
      padding: 20px;
      background: rgba(0, 0, 0, 0.85);
      color: white;
      font-family: 'Segoe UI', Tahoma, sans-serif;
      border-radius: 8px;
    }}
    .header {{
      font-size: 16px;
      font-weight: bold;
      margin-bottom: 12px;
      color: #60a5fa;
    }}
    .content {{ font-size: 13px; color: #d1d5db; }}
  </style>
</head>
<body>
  <div class="header">{name}</div>
  <div class="content" id="content">Waiting for game…</div>

  <script src="http://localhost:45678/sdk/loloverlay.js"></script>
  <script src="app.js"></script>
</body>
</html>
"#,
        name = name
    );

    fs::write(dir.join("index.html"), html)?;

    let js = format!(
        r#"// Initialize SDK — appId must match manifest id
LOLOverlay.init({{ appId: '{name}', version: '1.0.0' }});

const contentEl = document.getElementById('content');

LOLOverlay.game.onPhaseChange(async (event) => {{
  console.log(`Phase: ${{event.previous}} -> ${{event.current}}`);

  if (event.current === 'InGame') {{
    const session = await LOLOverlay.game.getSession();
    if (session) {{
      contentEl.innerHTML = `
        <strong>Game active!</strong><br>
        Player: ${{session.localPlayer.summonerName}}<br>
        Champion: ${{session.localPlayer.championName}}
      `;
    }}
  }} else {{
    contentEl.textContent = 'Waiting for game…';
  }}
}});
"#,
        name = name
    );

    fs::write(dir.join("app.js"), js)?;

    let readme = format!(
        "# {name}\n\nA broken-latch overlay app.\n\n## Development\n\n```bash\nblatch dev\n```\n\n## Package\n\n```bash\nblatch package\n```\n",
        name = name
    );
    fs::write(dir.join("README.md"), readme)?;

    Ok(())
}
