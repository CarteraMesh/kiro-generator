mod agent;
mod commands;
mod generator;
pub(crate) mod merging_format;
mod os;
mod schema;
use {
    crate::{generator::Generator, os::Fs},
    clap::Parser,
    tracing::debug,
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
            .without_time()
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
    let cmd = cli.command();
    if matches!(cmd, commands::Command::Version) {
        println!("{}", clap::crate_version!());
        return Ok(());
    }
    init_tracing(cli.debug);
    let span = tracing::info_span!(
        "main",
        dry_run = tracing::field::Empty,
        local_mode = tracing::field::Empty
    );
    let _guard = span.enter();

    let local_mode = cli.is_local(&cmd);
    let global_mode = cli.is_global(&cmd);
    if local_mode && global_mode {
        return Err(eyre::eyre!("Cannot use both local and global modes"));
    }
    let (home_dir, home_config) = cli.config()?;
    if global_mode {
        debug!(
            "changing working directory to {}",
            home_dir.as_os_str().display()
        );
        std::env::set_current_dir(&home_dir)?;
    }
    if local_mode {
        span.record("local_mode", true);
    }
    let dry_run = cli.dry_run(&cmd);
    if dry_run {
        span.record("dry_run", true);
    }

    let fs = Fs::new();
    let q_generator_config: Generator = if local_mode {
        Generator::new(fs, false, None)?
    } else {
        Generator::new(fs, global_mode, Some(home_config))?
    };
    debug!(
        "Loaded Agent Generator Config:\n{}",
        serde_json::to_string_pretty(&q_generator_config)?
    );

    match cmd {
        commands::Command::Validate(_args) | commands::Command::Generate(_args) => {
            q_generator_config.write_all(dry_run, local_mode).await?;
        }
        _ => {}
    };

    Ok(())
}
