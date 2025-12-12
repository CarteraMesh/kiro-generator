mod agent;
mod commands;
mod generator;
pub(crate) mod merging_format;
mod os;
mod schema;
use {
    crate::{
        generator::{AgentResult, Generator},
        os::Fs,
    },
    clap::Parser,
    super_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, *},
    tracing::debug,
    tracing_error::ErrorLayer,
    tracing_subscriber::prelude::*,
};
pub type Result<T> = color_eyre::Result<T>;

pub const DEFAULT_AGENT_RESOURCES: &[&str] = &["file://AGENTS.md", "file://README.md"];

fn init_tracing(debug: bool) {
    let filter = if debug {
        tracing_subscriber::EnvFilter::new("debug")
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
    };

    if debug {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_level(true)
                    .with_target(true),
            )
            .with(ErrorLayer::default())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_target(false)
                    .with_level(true),
            )
            .with(ErrorLayer::default())
            .init();
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
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
    // let global_config: Option<PathBuf> = if global_mode {
    //     Some(home_config)
    // } else if fs.exists(&local_config) {
    //     None
    // }
    let q_generator_config: Generator = if local_mode {
        Generator::local(fs)?
    } else {
        Generator::new(fs, Some(home_config))?
    };
    debug!(
        "Loaded Agent Generator Config:\n{}",
        serde_json::to_string_pretty(&q_generator_config)?
    );

    match cmd {
        commands::Command::Validate(_args) | commands::Command::Generate(_args) => {
            let results = q_generator_config.write_all(dry_run).await?;
            let writable: Vec<AgentResult> = if results.iter().any(|a| a.local) {
                results.into_iter().filter(|a| a.local).collect()
            } else {
                results
                    .into_iter()
                    .filter(|a| a.writable || a.agent.skeleton())
                    .collect()
            };
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec!["Agent", "Location"])
                .add_rows(writable);
            print!("{table}");
        }
        _ => {}
    };

    Ok(())
}
