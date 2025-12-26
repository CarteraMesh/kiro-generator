use {
    super::*,
    crate::kdl::{GeneratorConfig, KdlAgent},
    knuffel::parse,
    std::{fmt::Display, ops::Deref, path::Path},
};

fn add_local(
    fs: &Fs,
    agent: String,
    raw: String,
    agent_sources: &mut Vec<AgentSource>,
    local_agents: &mut HashSet<String>,
) {
    agent_sources.push(AgentSource::LocalInline(raw));
    let (local, empty) = AgentSource::new_local_agent(&agent, fs);
    if !empty {
        agent_sources.push(local);
        local_agents.insert(agent.clone());
    }
}

/// First pass: Discover all agents from configuration files
///
/// merge agent config from lowest precedence to higher precedence:
/// ```text
/// * `~/.kiro/generators/<agent-name>.toml`
/// * `~/.kiro/generators/kg.toml`
/// * `.kiro/generators/<agent-name>.toml`
/// * `.kiro/generators/kg.toml`
/// ```
#[tracing::instrument(name = "discover", level = "info")]
pub(super) fn agents(
    fs: &Fs,
    location: &ConfigLocation,
    format: &crate::output::OutputFormat,
) -> Result<(HashMap<String, KgAgent>, HashSet<String>)> {
    location.is_valid(fs)?;

    let global_path = location.global_kg();
    let local_path = location.local_kg();
    let global_exists = fs.exists(&global_path);

    let builder = Config::builder().add_source(
        config::File::from(global_path.clone())
            .required(false)
            .format(config::FileFormat::Toml),
    );
    let global_agents: KgConfig = builder
        .build()
        .wrap_err_with(|| format!("could not process global config: {}", global_path.display()))?
        .try_deserialize()
        .wrap_err_with(|| {
            format!(
                "could not deserialize global path {}",
                global_path.display()
            )
        })?;

    let local_config: KgConfig = Config::builder()
        .add_source(
            config::File::from(local_path.clone())
                .required(false)
                .format(config::FileFormat::Toml),
        )
        .build()
        .wrap_err_with(|| format!("could not process local path {}", local_path.display()))?
        .try_deserialize()
        .wrap_err_with(|| format!("could not deserialize local path {}", local_path.display()))?;

    let mut local_agents = HashSet::from_iter(local_config.agents.keys().cloned());
    tracing::debug!("found {} local agents", local_agents.len());
    let mut all_agents_names: HashSet<String> =
        HashSet::with_capacity(global_agents.agents.keys().len() + local_agents.len());
    all_agents_names.extend(local_agents.clone());
    all_agents_names.extend(global_agents.agents.keys().cloned());

    let mut resolved_agents: HashMap<String, KgAgent> =
        HashMap::with_capacity(all_agents_names.len());

    let global_dir = if global_exists {
        global_path
            .parent()
            .ok_or_else(|| {
                eyre!(
                    "global path does not have parent directory {}",
                    global_path.display()
                )
            })?
            .to_path_buf()
    } else {
        PathBuf::default()
    };
    let mut sources: HashMap<String, Vec<AgentSource>> = HashMap::new();
    for name in all_agents_names {
        sources.insert(name.clone(), Vec::with_capacity(4));
    }
    for (name, agent_sources) in sources.iter_mut() {
        let span = tracing::debug_span!("agent", name = ?name);
        let _enter = span.enter();
        tracing::trace!("matching location");
        match location {
            ConfigLocation::Local => {
                add_local(
                    fs,
                    name.to_string(),
                    local_config.get(name)?,
                    agent_sources,
                    &mut local_agents,
                );
            }
            ConfigLocation::Global(_) => {
                // ~/.kiro/generators/<agent-name>.toml
                agent_sources.push(AgentSource::GlobalFile(
                    global_dir.join(format!("{name}.toml")),
                ));
                // ~/.kiro/generators/kg.toml
                agent_sources.push(AgentSource::GlobalInline(global_agents.get(name)?));
            }
            ConfigLocation::Both(_) => {
                // ~/.kiro/generators/<agent-name>.toml
                agent_sources.push(AgentSource::GlobalFile(
                    global_dir.join(format!("{name}.toml")),
                ));
                // ~/.kiro/generators/kg.toml
                agent_sources.push(AgentSource::GlobalInline(global_agents.get(name)?));
                add_local(
                    fs,
                    name.to_string(),
                    local_config.get(name)?,
                    agent_sources,
                    &mut local_agents,
                );
            }
        };
    }

    for (name, agent_sources) in sources.iter() {
        let span = tracing::debug_span!("agent", name = ?name);
        let _enter = span.enter();
        let mut builder = Config::builder();
        for s in agent_sources {
            builder = builder.add_source(s.to_source(fs));
        }
        let mut agent: KgAgent = builder
            .build()?
            .try_deserialize()
            .wrap_err_with(|| format!("failed to deserialize {name}"))?;
        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                "Deserialized agent: {:?}",
                serde_json::to_string(&agent).unwrap()
            );
        }
        agent.name = name.clone();
        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                "{}",
                serde_json::to_string_pretty(&agent)
                    .ok()
                    .unwrap_or_default()
            );
        }
        resolved_agents.insert(name.clone(), agent);
    }
    Ok((resolved_agents, local_agents))
}

pub fn load_inline(fs: &Fs, path: impl AsRef<Path>) -> Result<GeneratorConfig> {
    if fs.exists(&path) {
        let content = fs
            .read_to_string_sync(&path)
            .wrap_err_with(|| format!("failed to read path '{}'", path.as_ref().display()))?;
        match parse(&format!("{}", path.as_ref().display()), &content) {
            Ok(c) => Ok(c),
            Err(e) => {
                let err_msg = e.to_string();
                eprintln!("{:?}", miette::Report::new(e));
                Err(eyre!("failed to parse: {err_msg}"))
            }
        }
    } else {
        Ok(GeneratorConfig::default())
    }
}

fn process_local(
    fs: &Fs,
    name: impl AsRef<str>,
    location: &ConfigLocation,
    inline: Option<&KdlAgent>,
    sources: &mut Vec<KdlAgentSource>,
) -> Result<KdlAgent> {
    let local_agent_path = location.local(&name);
    let result = KdlAgent::from_path(fs, &name, &local_agent_path)?;
    match result {
        None => Ok(KdlAgent::new(name)),
        Some(a) => {
            sources.push(KdlAgentSource::LocalFile(local_agent_path));
            if let Some(i) = inline {
                sources.push(KdlAgentSource::LocalInline);
                Ok(a.merge(i.clone()))
            } else {
                Ok(a)
            }
        }
    }
}

#[derive(Clone)]
pub struct ResolvedAgents {
    pub agents: HashMap<String, KdlAgent>,
    pub sources: KdlSources,
}

impl Deref for ResolvedAgents {
    type Target = HashMap<String, KdlAgent>;

    fn deref(&self) -> &Self::Target {
        &self.agents
    }
}

impl Debug for ResolvedAgents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "resolved={}", self.agents.len())
    }
}

impl Display for ResolvedAgents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "resolved={}", self.agents.len())
    }
}

/// First pass: Discover all agents from configuration files
///
/// merge agent config from lowest precedence to higher precedence:
/// ```text
/// * `~/.kiro/generators/<agent-name>.toml`
/// * `~/.kiro/generators/kg.toml`
/// * `.kiro/generators/<agent-name>.toml`
/// * `.kiro/generators/kg.toml`
/// ```
#[tracing::instrument(level = "info")]
pub fn discover(
    fs: &Fs,
    location: &ConfigLocation,
    format: &crate::output::OutputFormat,
) -> Result<ResolvedAgents> {
    location.is_valid(fs)?;

    let global_path = location.global_kg();
    let local_path = location.local_kg();
    let global_agents: GeneratorConfig = load_inline(fs, global_path)?;
    let local_agents: GeneratorConfig = load_inline(fs, local_path)?;
    tracing::debug!("found {} local agents", local_agents.agents.len());

    let local_names = local_agents.names();
    let global_names = global_agents.names();
    let mut all_agents_names: HashSet<String> =
        HashSet::with_capacity(global_names.len() + local_names.len());
    all_agents_names.extend(local_names.clone());
    all_agents_names.extend(global_names);

    let mut resolved_agents: HashMap<String, KdlAgent> =
        HashMap::with_capacity(all_agents_names.len());
    let mut sources: KdlSources = KdlSources::from(&all_agents_names);
    for (name, agent_sources) in sources.iter_mut() {
        let span = tracing::debug_span!("agent", name = ?name);
        let _enter = span.enter();
        tracing::trace!("matching location");

        match location {
            ConfigLocation::Local => {
                resolved_agents.insert(
                    name.to_string(),
                    process_local(fs, name, location, local_agents.get(name), agent_sources)?,
                );
            }
            ConfigLocation::Both(_) => {
                let mut result =
                    process_local(fs, name, location, local_agents.get(name), agent_sources)?;
                if let Some(a) = global_agents.get(name) {
                    agent_sources.push(KdlAgentSource::GlobalInline);
                    result = result.merge(a.clone());
                }
                let maybe_global_file = KdlAgent::from_path(fs, name, location.global(name))?;
                if let Some(global) = maybe_global_file {
                    agent_sources.push(KdlAgentSource::GlobalFile(location.global(name)));
                    result = result.merge(global.clone());
                }
                resolved_agents.insert(name.to_string(), result);
            }
            ConfigLocation::Global(_) => {
                let mut global_file = match KdlAgent::from_path(fs, name, location.global(name))? {
                    None => KdlAgent::new(name),
                    Some(a) => {
                        agent_sources.push(KdlAgentSource::GlobalFile(location.global(name)));
                        a
                    }
                };
                if let Some(inline) = global_agents.get(name) {
                    agent_sources.push(KdlAgentSource::GlobalInline);
                    global_file = global_file.merge(inline.clone());
                }
                resolved_agents.insert(name.to_string(), global_file);
            }
        };
    }
    if let Err(e) = format.sources(&sources) {
        tracing::error!("Failed to format sources: {}", e);
    }
    Ok(ResolvedAgents {
        agents: resolved_agents,
        sources,
    })
}

#[cfg(test)]
mod tests {
    use {super::*, crate::os::ACTIVE_USER_HOME, std::path::PathBuf};

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_local_agents_kdl() -> Result<()> {
        let fs = Fs::new();
        let resolved = discover(
            &fs,
            &ConfigLocation::Local,
            &crate::output::OutputFormat::Table(true),
        )?;
        let agents = resolved.agents;
        let sources = resolved.sources;
        assert!(!agents.is_empty());
        assert_eq!(sources.keys().len(), 3);
        assert!(sources.contains_key("base"));
        assert!(sources.contains_key("aws-test"));
        assert!(sources.contains_key("dependabot"));

        let source = sources.get("base").unwrap();
        assert_eq!(source.len(), 2);

        let source = sources.get("aws-test").unwrap();
        assert_eq!(source.len(), 2);

        let source = sources.get("dependabot").unwrap();
        assert_eq!(source.len(), 2);

        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        discover(
            &fs,
            &ConfigLocation::Global(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_global_agents_kdl() -> Result<()> {
        let fs = Fs::new();
        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let resolved = discover(
            &fs,
            &ConfigLocation::Global(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;
        assert_eq!(resolved.len(), 3);
        for agent_sources in resolved.sources.values() {
            for s in agent_sources {
                assert!(
                    matches!(
                        s,
                        KdlAgentSource::GlobalInline | KdlAgentSource::GlobalFile(_)
                    ),
                    "agent is not global"
                )
            }
        }
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_both_agents_kdl() -> Result<()> {
        let fs = Fs::new();
        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let resolved = discover(
            &fs,
            &ConfigLocation::Both(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;

        assert_eq!(resolved.len(), 3);

        for (n, agent_sources) in resolved.sources.iter() {
            if n == "aws-test" {
                assert_eq!(agent_sources.len(), 4);
            } else {
                assert_eq!(agent_sources.len(), 3);
            }
        }
        Ok(())
    }
}
