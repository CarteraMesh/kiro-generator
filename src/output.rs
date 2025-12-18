use {
    crate::{
        Result,
        agent::{Agent, KgAgent},
        generator::AgentResult,
        os::Fs,
        source::AgentSource,
    },
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

    fn agent_result_to_row(&self, result: &AgentResult) -> Row {
        let mut row = Row::new();

        // Agent name with skeleton indicator
        let name_cell = if result.agent.skeleton() {
            Cell::new(format!("{} {}", result.agent.name, "💀"))
        } else {
            Cell::new(&result.agent.name)
        };
        row.add_cell(name_cell);

        // Location: 🏠 for global, 📁 for local
        let location = if result.agent.skeleton() {
            Cell::new("")
        } else if result.destination.is_absolute() {
            Cell::new("🏠")
        } else {
            Cell::new("📁")
        };
        row.add_cell(location);

        // MCP servers (only enabled ones)
        let mut servers = Vec::new();
        for (k, v) in &result.agent.mcp_servers.mcp_servers {
            if !v.disabled {
                servers.push(k.clone());
            }
        }
        row.add_cell(Cell::new(servers.join(", ")));

        // Allowed tools
        let allowed_tools: Vec<String> = result
            .agent
            .allowed_tools
            .0
            .iter()
            .filter(|t| !t.is_empty())
            .cloned()
            .collect();
        let mut enabled_tools = Vec::with_capacity(allowed_tools.len());
        for t in allowed_tools {
            if t.len() < 2 {
                continue;
            }
            if let Some(server_name) = t.strip_prefix("@") {
                match result.agent.mcp_servers.mcp_servers.get(server_name) {
                    Some(mcp) if !mcp.disabled => {} // enabled, keep it
                    _ => continue,                   // disabled or doesn't exist, skip it
                }
            }
            enabled_tools.push(t);
        }
        row.add_cell(Cell::new(enabled_tools.join(", ")));

        // Forced permissions (security-critical)
        let sh = result
            .agent
            .get_tool_shell()
            .force_allowed_commands
            .0
            .iter()
            .cloned()
            .collect::<Vec<String>>();
        let read = result
            .agent
            .get_tool_read()
            .force_allowed_paths
            .0
            .iter()
            .cloned()
            .collect::<Vec<String>>();
        let write = result
            .agent
            .get_tool_write()
            .force_allowed_paths
            .0
            .iter()
            .cloned()
            .collect::<Vec<String>>();

        let mut forced = vec![];
        if !sh.is_empty() {
            let l = serde_yml::to_string(&sh).unwrap_or_default();
            forced.push(Cell::new(format!("cmds:\n{l}")));
        }
        if !read.is_empty() {
            let l = serde_yml::to_string(&read).unwrap_or_default();
            forced.push(Cell::new(format!("read:\n{l}")));
        }
        if !write.is_empty() {
            let l = serde_yml::to_string(&write).unwrap_or_default();
            forced.push(Cell::new(format!("write:\n{l}")));
        }

        // resources
        let resources = serde_yml::to_string(&result.agent.resources.0).unwrap_or_default();
        row.add_cell(Cell::new(resources));
        match forced.len() {
            0 => {
                row.add_cell(Cell::new("").set_colspan(3));
            }
            1 => {
                row.add_cell(forced[0].clone().set_colspan(3));
            }
            2 => {
                row.add_cell(forced[0].clone());
                row.add_cell(forced[1].clone().set_colspan(2));
            }
            _ => {
                for c in forced {
                    row.add_cell(c);
                }
            }
        }

        row
    }

    pub fn result(
        &self,
        dry_run: bool,
        show_skeletons: bool,
        results: Vec<AgentResult>,
    ) -> Result<()> {
        match self {
            Format::Table => {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic);

                // Different header styling for dry-run vs actual generation
                if dry_run {
                    table.set_header(vec![
                        Cell::new(format!("Agent {} (PREVIEW)", emojis_rs::EMOJI_ROBOT))
                            .fg(Color::Yellow),
                        Cell::new("Loc").fg(Color::Yellow),
                        Cell::new(format!("MCP {}", emojis_rs::EMOJI_COMPUTER)).fg(Color::Yellow),
                        Cell::new(format!("Allowed Tools {}", emojis_rs::EMOJI_GEAR))
                            .fg(Color::Yellow),
                        Cell::new(format!("Resources {}", emojis_rs::EMOJI_DOCUMENT))
                            .fg(Color::Yellow),
                        Cell::new("Forced Permissions")
                            .set_colspan(3)
                            .set_alignment(CellAlignment::Center)
                            .fg(Color::Yellow),
                    ]);
                } else {
                    table.set_header(vec![
                        agent_header(),
                        Cell::new("Loc"),
                        Cell::new(format!("MCP {}", emojis_rs::EMOJI_COMPUTER)),
                        Cell::new(format!("Allowed Tools {}", emojis_rs::EMOJI_GEAR)),
                        Cell::new(format!("Resources {}", emojis_rs::EMOJI_DOCUMENT)),
                        Cell::new("Forced Permissions")
                            .set_colspan(3)
                            .set_alignment(CellAlignment::Center),
                    ]);
                }

                for result in &results {
                    if show_skeletons || !result.agent.skeleton() {
                        table.add_row(self.agent_result_to_row(result));
                    }
                }

                println!("{table}");
                if dry_run {
                    println!("\n{} Config is valid", emojis_rs::EMOJI_SUCCESS);
                    println!(
                        "{} Run {} to generate agent files",
                        "→".yellow().bold(),
                        "kg generate".green().bold()
                    );
                } else {
                    println!("\n{} Generated agent files", emojis_rs::EMOJI_CHECK);
                }
                Ok(())
            }
            Format::Json => {
                let kiro_agents: Vec<Agent> = results.into_iter().map(|a| a.kiro_agent).collect();
                println!("{}", serde_json::to_string_pretty(&kiro_agents)?);
                Ok(())
            }
        }
    }
}
