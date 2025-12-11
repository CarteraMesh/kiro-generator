use {
    clap::{Parser, Subcommand},
    std::path::PathBuf,
};

#[derive(Parser)]
#[command(name = "kg", long_version = clap::crate_version!(), about, long_about = "blasdh")]
pub struct Cli {
    #[arg(long, global = true, short = 'v', short_alias = 'd', aliases = ["verbose", "debug"], default_value = "false")]
    pub debug: bool,
    //  #[arg(short = 'V', long, help = "Print version")]
    //    pub version: bool,
    pub configs: Option<Vec<PathBuf>>,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(clap::Args, Clone, Default)]
pub struct Args {
    #[arg(
        long,
        help = "Ignore global $HOME directory, use only what's in  .kiro/generators/kg.toml"
    )]
    pub local: bool,
    #[arg(long, help = "Ignore local .kiro/generators/kg.toml config")]
    #[arg(short = 'g', long, help = "Print version")]
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

    pub fn configs(&self) -> (PathBuf, PathBuf) {
        let global = dirs::home_dir().map_or(
            PathBuf::from(".kiro").join("generators").join("kg.toml"),
            |dir| dir.join(".kiro").join("generators").join("kg.toml"),
        );
        let local = PathBuf::from(".kiro").join("generators").join("kg.toml");
        (global, local)
    }
}
