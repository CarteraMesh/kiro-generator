mod agent;
mod commands;
mod generator;
pub(crate) mod merging_format;
mod os;
mod schema;
use {
    crate::{generator::Generator, os::Fs},
    clap::Parser,
    tracing::{debug, info},
};
pub type Result<T> = eyre::Result<T>;
pub const DEFAULT_AGENT_RESOURCES: &[&str] = &["file://AGENTS.md", "file://README.md"];

fn init_tracing(debug: bool) {
    let filter = if debug {
        tracing_subscriber::EnvFilter::new("debug")
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
    };

    if debug {
        // Verbose format with timestamps and targets for debugging
        tracing_subscriber::fmt()
            .with_level(true)
            .with_target(true)
            .with_env_filter(filter)
            .init();
    } else {
        // Clean format for CLI - no timestamps, no targets
        tracing_subscriber::fmt()
            .with_target(false)
            .without_time()
            .with_level(true)
            .with_env_filter(filter)
            .init();
    }
}
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = commands::Cli::parse();
    init_tracing(cli.debug);
    let span = tracing::info_span!(
        "main",
        dry_run = tracing::field::Empty,
        local_mode = tracing::field::Empty
    );
    let _guard = span.enter();
    if cli.local {
        span.record("local_mode", true);
    }
    let (global, local) = cli.configs();

    let q_generator_config: Generator = if cli.local {
        Generator::new(local.clone(), Fs::new())?
    } else {
        Generator::new(global, Fs::new())?
    };
    debug!(
        "Loaded Agent Generator Config:\n{}",
        serde_json::to_string_pretty(&q_generator_config)?
    );

    let mut dry_run = false;
    match cli.command {
        commands::Commands::Validate => {
            span.record("dry_run", true);
            dry_run = true;
            info!("Validating agent generator config");
        }
        commands::Commands::Generate => {
            info!("Overwriting existing config");
        }
    };
    q_generator_config.write_all(dry_run).await?;
    Ok(())
}
