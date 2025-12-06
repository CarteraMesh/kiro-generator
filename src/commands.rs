use {
    clap::{Parser, Subcommand},
    std::path::PathBuf,
};

#[derive(Parser)]
#[command(name = "kg")]
pub struct Cli {
    #[arg(long, short = 'v', alias = "verbose", default_value = "false")]
    pub debug: bool,
    #[arg(long, help = "Output to directory instead of default")]
    pub output: Option<PathBuf>,
    #[arg(
        long,
        help = "Ignore global $HOME directory, use only what's in  .kiro/generators/kg.toml"
    )]
    pub local: bool,
    pub configs: Option<Vec<PathBuf>>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(alias = "v")]
    Validate,
    #[command(alias = "g")]
    Generate,
}

impl Cli {
    pub fn configs(&self) -> (PathBuf, PathBuf) {
        let global = dirs::home_dir().map_or(
            PathBuf::from(".kiro").join("generators").join("kg.toml"),
            |dir| dir.join(".kiro").join("generators").join("kg.toml"),
        );
        let local = PathBuf::from(".kiro").join("generators").join("kg.toml");
        (global, local)
    }
}
