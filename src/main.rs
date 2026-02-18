use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use rust_analyzer_server::RustAnalyzerMCPServer;

#[derive(Parser)]
#[command(name = "rust-analyzer-server", about = "Standalone HTTP server for rust-analyzer")]
struct Cli {
    /// Workspace path (defaults to current directory)
    #[arg(short, long)]
    workspace: Option<PathBuf>,

    /// Port to listen on
    #[arg(short, long, default_value = "15423", env = "RUST_ANALYZER_PORT")]
    port: u16,

    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install Claude Code skills into a target project
    Install {
        /// Target project path
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install { path }) => {
            let target = path.canonicalize().unwrap_or(path);
            rust_analyzer_server::install::install_skills(&target)?;
        }
        None => {
            let workspace = cli
                .workspace
                .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));
            let server = RustAnalyzerMCPServer::with_workspace(workspace);
            rust_analyzer_server::http::serve(&cli.bind, cli.port, server).await?;
        }
    }

    Ok(())
}
