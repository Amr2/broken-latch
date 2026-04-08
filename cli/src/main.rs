use clap::{Parser, Subcommand};

mod commands;
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

    /// Start development server with file watching
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
