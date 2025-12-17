use {
    crate::{Result, agent::KgAgent, generator::AgentResult, os::Fs, source::AgentSource},
    colored::Colorize,
    std::{collections::HashMap, fmt::Display},
    super_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, *},
};
/// Override the color setting. Default is [`ColorOverride::Auto`].
#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum ColorOverride {
    /// Always display color (i.e. force it).
    Always,
    /// Automatically determine if color should be used or not.
    Auto,
    /// Never display color.
    Never,
}

#[derive(Copy, Clone, Default, Debug, clap::ValueEnum)]
pub enum Format {
    #[default]
    Table,
    Json,
}
impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Table => write!(f, "table"),
            Format::Json => write!(f, "json"),
        }
    }
}

pub(crate) fn agent_header() -> Cell {
    Cell::new(format!("Agent {}", emojis_rs::EMOJI_ROBOT))
}

impl Format {
    pub fn trace_agent(&self, agent: &KgAgent) -> Result<()> {
        match self {
            Format::Json | Format::Table => eprintln!("{}", serde_json::to_string_pretty(agent)?),
        };
        Ok(())
    }

    pub fn sources(&self, fs: &Fs, sources: &HashMap<String, Vec<AgentSource>>) -> Result<()> {
        match self {
            Format::Table => {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        agent_header(),
                        Cell::new("Sources")
                            .set_colspan(4)
                            .set_alignment(CellAlignment::Center),
                    ]);
                for (name, agent_sources) in sources.iter() {
                    let mut row: Vec<Cell> = vec![Cell::new(name.to_string())];
                    row.extend(agent_sources.iter().map(|s| s.to_cell(fs)));
                    table.add_row(row);
                }
                eprintln!("{table}");
                Ok(())
            }
            Format::Json => Ok(()),
        }
    }

    pub fn result(&self, dry_run: bool, results: Vec<AgentResult>) -> Result<()> {
        match self {
            Format::Table => {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        agent_header(),
                        Cell::new(format!("Location {}", emojis_rs::EMOJI_HARD_DISK)),
                        Cell::new(format!("Inherits {}", emojis_rs::EMOJI_RUST)),
                        Cell::new(format!("MCP {}", emojis_rs::EMOJI_COMPUTER)),
                        Cell::new("Forced")
                            .set_colspan(3)
                            .set_alignment(CellAlignment::Center),
                    ])
                    .add_rows(results);
                println!("{table}");
                if dry_run {
                    println!("Config is {} {}", "valid".green(), emojis_rs::EMOJI_SUCCESS);
                } else {
                    println!("Overwrote existing config {}", emojis_rs::EMOJI_CHECK);
                }
                Ok(())
            }
            Format::Json => {
                // Implement JSON output logic here
                Ok(())
            }
        }
    }
}
