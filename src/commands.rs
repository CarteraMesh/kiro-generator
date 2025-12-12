use {
    clap::{Parser, Subcommand},
    color_eyre::eyre::eyre,
    std::path::PathBuf,
};

#[derive(Parser)]
#[command(name = "kg", long_version = clap::crate_version!(), about, long_about = "")]
pub struct Cli {
    #[arg(long, global = true, short = 'v', short_alias = 'd', aliases = ["verbose", "debug"], default_value = "false")]
    pub debug: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(clap::Args, Clone, Default)]
pub struct Args {
    #[arg(
        long,
        conflicts_with = "global",
        help = "Ignore global $HOME directory, use only what's in  .kiro/generators/kg.toml"
    )]
    pub local: bool,
    #[arg(
        short = 'g',
        long,
        conflicts_with = "local",
        help = "Ignore local .kiro/generators/kg.toml config"
    )]
    pub global: bool,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    #[command(alias = "v")]
    Validate(Args),
    #[command(alias = "g")]
    Generate(Args),
    #[command(alias = "m")]
    Migrate,
    Version,
}

impl Default for Command {
    fn default() -> Self {
        Command::Validate(Args::default())
    }
}

impl Cli {
    pub fn command(&self) -> Command {
        match &self.command {
            Some(command) => command.clone(),
            None => Default::default(),
        }
    }

    pub fn dry_run(&self, command: &Command) -> bool {
        matches!(command, Command::Validate(_))
    }

    pub fn is_local(&self, command: &Command) -> bool {
        match command {
            Command::Generate(args) => args.local,
            Command::Validate(args) => args.local,
            _ => false,
        }
    }

    pub fn is_global(&self, command: &Command) -> bool {
        match command {
            Command::Generate(args) => args.global,
            Command::Validate(args) => args.global,
            _ => false,
        }
    }

    /// Return home dir and ~/.kiro/generators/kg.toml
    pub fn config(&self) -> crate::Result<(PathBuf, PathBuf)> {
        let home_dir = dirs::home_dir().ok_or(eyre!("cannot locate home directory"))?;
        let cfg = home_dir.join(".kiro").join("generators").join("kg.toml");
        Ok((home_dir, cfg))
    }
}
