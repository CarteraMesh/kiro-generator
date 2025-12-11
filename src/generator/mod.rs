use {
    crate::{
        agent::{
            MergingAwsTool,
            MergingExecuteShellTool,
            MergingReadTool,
            MergingWriteTool,
            QgAgent,
            ToolMerge,
            ToolTarget,
        },
        merging_format::MergingTomlFormat,
        os::Fs,
    },
    config::Config,
    eyre::{Context, eyre},
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet},
        path::PathBuf,
    },
};

/// Container for all agent declarations from kg.toml files
#[derive(Debug, Default, Deserialize)]
struct KgConfig {
    #[serde(default)]
    agents: HashMap<String, serde_json::Value>,
}

/// Main generator that orchestrates agent discovery and merging
#[derive(Debug, Serialize)]
pub struct Generator {
    global_path: PathBuf,
    agents: HashMap<String, QgAgent>,
    local_agents: HashSet<String>, // Agents defined in local kg.toml
    #[serde(skip)]
    fs: Fs,
}

impl Generator {
    /// Read kg.toml file and collect all agents
    pub fn new(fs: Fs, ignore_local: bool, global_path: Option<PathBuf>) -> eyre::Result<Self> {
        if let Some(ref p) = global_path {
            tracing::debug!("global config set to {}", p.as_os_str().display());
            if !p.is_absolute() {
                return Err(eyre!(
                    "global path should be absolute, I received {}",
                    p.as_os_str().display()
                ));
            }
        }
        let global_path = global_path.unwrap_or_default();
        let span = tracing::info_span!(
            "discover",
            ignore = ignore_local,
            global = fs.exists(&global_path)
        );
        let _guard = span.enter();
        let (agents, local_agents) = Self::discover_agents(&fs, &global_path, ignore_local)?;
        Ok(Self {
            global_path,
            agents,
            local_agents,
            fs,
        })
    }

    /// First pass: Discover all agents from ~/.kiro/generators/kg.toml and
    /// local .kiro/generators/kg.yaml files
    ///
    /// merge agent config from lowest precedence to higher precedence:
    /// ```text
    /// * `~/.kiro/generators/<agent-name>.toml`
    /// * `~/.kiro/generators/kg.toml`
    /// * `.kiro/generators/<agent-name>.toml`
    /// * `.kiro/generators/kg.toml`
    /// ```
    fn discover_agents(
        fs: &Fs,
        global_path: &PathBuf,
        ignore_local: bool,
    ) -> eyre::Result<(HashMap<String, QgAgent>, HashSet<String>)> {
        let mut local_path: PathBuf = PathBuf::from(".kiro").join("generators").join("kg.toml");
        if ignore_local {
            local_path = PathBuf::default();
        } else if fs.is_same(global_path, &local_path) {
            tracing::debug!("global and local are the same");
            local_path = PathBuf::default();
        }
        if !fs.exists(&local_path) && !fs.exists(global_path) {
            return Err(eyre!(
                "no kg.toml configuration found in local or global locations \
                 (ignore_local={ignore_local})",
            ));
        }
        if !ignore_local && fs.exists(&local_path) {
            tracing::debug!("Loading local kg.toml: {}", local_path.display());
        }
        let builder = Config::builder().add_source(
            config::File::from(global_path.clone())
                .required(false)
                .format(config::FileFormat::Toml),
        );
        let global_agents: KgConfig = builder
            .build()
            .context(format!(
                "could not process global config: {}",
                global_path.display()
            ))?
            .try_deserialize()
            .context(format!(
                "could deserialize global path {}",
                global_path.as_os_str().display()
            ))?;
        let local_config: KgConfig = if ignore_local {
            KgConfig::default()
        } else {
            // Parse local kg.toml with MergingTomlFormat to get agent names
            let local_config: KgConfig = Config::builder()
                .add_source(
                    config::File::from(local_path.clone())
                        .required(false)
                        .format(config::FileFormat::Toml),
                )
                .build()
                .context(format!(
                    "could not process local path {}",
                    local_path.as_os_str().display()
                ))?
                .try_deserialize()
                .context(format!(
                    "could deserialize local path {}",
                    local_path.as_os_str().display()
                ))?;
            local_config
        };

        let mut local_agents = HashSet::from_iter(local_config.agents.keys().cloned());
        tracing::debug!("found {} local agents", local_agents.len());
        let mut all_agents_names: HashSet<String> =
            HashSet::with_capacity(global_agents.agents.keys().len() + local_agents.len());
        all_agents_names.extend(local_agents.clone());
        all_agents_names.extend(global_agents.agents.keys().cloned());

        let mut resolved_agents: HashMap<String, QgAgent> =
            HashMap::with_capacity(all_agents_names.len());

        let global_dir = if fs.exists(global_path) {
            match global_path.parent() {
                Some(parent) => parent.to_path_buf(),
                None => {
                    return Err(eyre!(
                        "global path does not have parent directory {}",
                        global_path.as_os_str().display()
                    ));
                }
            }
        } else {
            PathBuf::default()
        };
        for name in all_agents_names {
            let span = tracing::debug_span!("merge", agent = ?name);
            let _enter = span.enter();
            let mut builder = Config::builder();
            // ~/.kiro/generators/<agent-name>.toml
            let location = global_dir.join(format!("{name}.toml"));
            let content = fs.read_to_string_sync(&location).ok().unwrap_or_default();
            if !content.is_empty() {
                builder = builder.add_source(config::File::from_str(&content, MergingTomlFormat));
                tracing::debug!("adding {}", location.as_os_str().display());
            }

            // ~/.kiro/generators/kg.toml
            if let Some(a) = global_agents.agents.get(&name) {
                tracing::debug!("adding ~/.kiro/generators/kg.toml");
                builder = builder.add_source(config::File::from_str(
                    &toml::to_string(a)?,
                    MergingTomlFormat,
                ));
            }
            if !ignore_local {
                // local .kiro/generators/<agent-name>.toml
                let location = PathBuf::from(".kiro")
                    .join("generators")
                    .join(format!("{name}.toml"));
                let content = fs.read_to_string_sync(&location).ok().unwrap_or_default();
                if !content.is_empty() {
                    builder =
                        builder.add_source(config::File::from_str(&content, MergingTomlFormat));
                    tracing::debug!("adding {}", location.as_os_str().display());
                    local_agents.insert(name.clone());
                }
            }

            // .kiro/generators/kg.toml
            if let Some(a) = local_config.agents.get(&name) {
                tracing::debug!("adding local .kiro/generators/kg.toml");
                builder = builder.add_source(config::File::from_str(
                    &toml::to_string(a)?,
                    MergingTomlFormat,
                ));
            }
            let agent: QgAgent = builder
                .build()?
                .try_deserialize()
                .context(format!("failed to deserialize {name}"))?;
            resolved_agents.insert(name, agent);
        }
        Ok((resolved_agents, local_agents))
    }

    /// Merge all agents with their inheritance chains
    pub fn merge(&self) -> eyre::Result<Vec<QgAgent>> {
        let mut resolved_agents: HashMap<String, QgAgent> =
            HashMap::with_capacity(self.agents.len());

        let mut cached_serialized_agents: HashMap<String, String> =
            HashMap::with_capacity(self.agents.len());
        for (k, v) in self.agents.iter() {
            let value = serde_json::to_value(v)?;
            if value.is_null() {
                tracing::warn!("agent {k} is empty");
                continue;
            }
            cached_serialized_agents.insert(
                k.clone(),
                toml::to_string(&v).context(format!("could not serialize agent {k} to toml"))?,
            );
        }

        for (name, inline_agent) in &self.agents {
            let span = tracing::debug_span!("merge", parents = ?inline_agent.inherits.0.len(), child = ?name);
            let _enter = span.enter();
            let mut builder = Config::builder();
            if !cached_serialized_agents.contains_key(name) {
                return Err(eyre::format_err!(
                    "Cached source for agent '{name}' not found",
                ));
            }
            for parent in &inline_agent.inherits.0 {
                if !cached_serialized_agents.contains_key(parent) {
                    return Err(eyre::format_err!(
                        "Cached source for parent agent '{parent}' not found",
                    ));
                }
                let parent_source = cached_serialized_agents.get(parent).unwrap();
                builder =
                    builder.add_source(config::File::from_str(parent_source, MergingTomlFormat));
            }
            let source = cached_serialized_agents.get(name).unwrap();
            builder = builder.add_source(config::File::from_str(source, MergingTomlFormat));

            let mut agent: QgAgent = builder.build()?.try_deserialize().context(format!(
                "failed to merge agent {name} with parents {:?}",
                inline_agent.inherits.0
            ))?;
            //            self.resolve_agent(name, &global_dir, &local_dir, &mut
            // resolved_agents)?; Set the name on the agent
            for parent in &inline_agent.inherits.0 {
                if !self.agents.contains_key(parent) {
                    return Err(eyre::format_err!(
                        "[{name}] Parent agent definition '{parent}' not found",
                    ));
                }
                agent = self.merge_agents(self.agents.get(parent).unwrap(), agent)?;
            }

            agent.name = name.clone();
            resolved_agents.insert(name.clone(), agent);
        }

        // Filter out skeletons and return QgAgent instances
        let agents: Vec<QgAgent> = resolved_agents
            .into_iter()
            .filter(|(_, agent)| !agent.skeleton)
            .map(|(_, agent)| agent)
            .collect();

        Ok(agents)
    }

    /// Merge parent into child (child takes precedence)
    fn merge_agents(&self, parent: &QgAgent, mut child: QgAgent) -> eyre::Result<QgAgent> {
        let span = tracing::debug_span!("merge-tools", parent = ?parent, child = ?child);
        let _enter = span.enter();

        let agent_aws: MergingAwsTool = child.get_tool(ToolTarget::Aws);
        let parent_aws: MergingAwsTool = parent.get_tool(ToolTarget::Aws);
        let agent_exec: MergingExecuteShellTool = child.get_tool(ToolTarget::Shell);
        let parent_exec: MergingExecuteShellTool = parent.get_tool(ToolTarget::Shell);
        let agent_fs_read: MergingReadTool = child.get_tool(ToolTarget::Read);
        let parent_fs_read: MergingReadTool = parent.get_tool(ToolTarget::Read);
        let agent_fs_write: MergingWriteTool = child.get_tool(ToolTarget::Write);
        let parent_fs_write: MergingWriteTool = parent.get_tool(ToolTarget::Write);

        child.set_tool(ToolTarget::Shell, agent_exec.merge(parent_exec));
        child.set_tool(ToolTarget::Aws, agent_aws.merge(parent_aws));
        child.set_tool(ToolTarget::Read, agent_fs_read.merge(parent_fs_read));
        child.set_tool(ToolTarget::Write, agent_fs_write.merge(parent_fs_write));

        Ok(child)
    }

    /// Check if an agent is defined in local kg.toml
    pub fn is_local(&self, agent_name: impl AsRef<str>) -> bool {
        self.local_agents.contains(agent_name.as_ref())
    }

    /// Get the destination directory for an agent (global or local)
    pub fn destination_dir(&self, agent_name: impl AsRef<str>) -> PathBuf {
        if self.is_local(agent_name) {
            PathBuf::from(".kiro").join("agents")
        } else {
            dirs::home_dir()
                .map(|h| h.join(".kiro").join("agents"))
                .unwrap_or_else(|| PathBuf::from(".kiro").join("agents"))
        }
    }

    pub async fn write_all(&self, dry_run: bool, local_mode: bool) -> eyre::Result<Vec<QgAgent>> {
        if dry_run {
            tracing::info!("Validating config");
        } else {
            tracing::info!("Overwriting existing config");
        }
        let mut agents = self.merge()?;
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        for agent in &agents {
            let span = tracing::info_span!("writer", agent = agent.name.as_str());
            let _guard = span.enter();
            self.write(agent, dry_run, local_mode).await?;
        }
        Ok(agents)
    }

    pub async fn write(
        &self,
        agent: &QgAgent,
        dry_run: bool,
        local_mode: bool,
    ) -> eyre::Result<()> {
        let destination = self.destination_dir(&agent.name);
        let inherits = agent
            .inherits
            .iter()
            .map(|a| a.as_str())
            .collect::<Vec<_>>();
        tracing::info!(
            "Location {} inherits [{}]",
            destination.display(),
            inherits.join(",")
        );
        if dry_run {
            return Ok(());
        }

        if !self.fs.exists(&destination) {
            self.fs.create_dir_all(&destination).await?;
        }
        if local_mode && !destination.is_absolute() {
            agent.write(&self.fs, destination).await?;
            return Ok(());
        }
        agent.write(&self.fs, destination).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::agent::{
            Agent,
            AwsTool,
            ExecuteShellTool,
            ToolTarget,
            WriteTool,
            hook::HookTrigger,
        },
    };

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_agents() -> eyre::Result<()> {
        let fs = Fs::new();
        match _discover_agents(fs.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                let dest = PathBuf::from(".kiro").join("agents");
                if fs.exists(&dest) {
                    let _dir = fs.read_dir(&dest).await?;
                    // TODO spit contents via tracing::error
                }
                Err(e)
            }
        }
    }

    async fn _discover_agents(fs: Fs) -> eyre::Result<()> {
        //        let global =
        // PathBuf::from(".kiro").join("generators").join("kg.toml");
        let generator = Generator::new(fs.clone(), false, None)?;
        assert!(!generator.agents.is_empty());
        assert_eq!(4, generator.agents.len());
        assert_eq!(4, generator.local_agents.len());
        // Check that base agent exists and is a skeleton
        if let Some(base) = generator.agents.get("base") {
            assert!(base.skeleton);
        }
        let agents = generator.write_all(false, true).await?;

        for agent in &agents {
            let destination = PathBuf::from(".kiro")
                .join("agents")
                .join(format!("{}.json", agent.name));
            tracing::info!("checking output at {}", destination.as_os_str().display());
            if agent.skeleton {
                assert!(!fs.exists(&destination));
            } else {
                assert!(fs.exists(&destination));
            }
        }
        let content = fs
            .read_to_string(PathBuf::from(".kiro").join("agents").join("aws-test.json"))
            .await?;
        let kiro_agent: Agent = serde_json::from_str(&content)?;
        assert_eq!("aws-test", kiro_agent.name);
        assert_eq!(
            "all the AWS tools you want",
            kiro_agent.description.clone().unwrap_or_default()
        );
        assert!(kiro_agent.model.is_none());
        assert_eq!(
            "you are an AWS expert",
            kiro_agent.prompt.clone().unwrap_or_default()
        );
        assert_eq!(1, kiro_agent.tools.len());
        assert!(kiro_agent.tools.contains("*"));
        tracing::info!("{:?}", kiro_agent.allowed_tools);
        assert_eq!(4, kiro_agent.allowed_tools.len());
        let allowed_tools = ["read", "knowledge", "@fetch", "@awsdocs"];
        for tool in allowed_tools {
            assert!(kiro_agent.allowed_tools.contains(tool));
        }
        tracing::info!("{:?}", kiro_agent.mcp_servers.mcp_servers.keys());
        assert_eq!(4, kiro_agent.mcp_servers.mcp_servers.len());
        for mcp in ["awsbilling", "awsdocs", "cargo", "rustdocs"] {
            assert!(kiro_agent.mcp_servers.mcp_servers.contains_key(mcp));
        }

        tracing::info!("{:?}", kiro_agent.resources);
        assert_eq!(3, kiro_agent.resources.len());
        for r in [
            "file://.amazonq/rules/**/*.md",
            "file://AGENTS.md",
            "file://README.md",
        ] {
            assert!(kiro_agent.resources.contains(r));
        }

        tracing::info!("{:?}", kiro_agent.tools_settings.keys());
        assert_eq!(4, kiro_agent.tools_settings.len());
        let aws_tool: AwsTool = kiro_agent.get_tool(ToolTarget::Aws);
        tracing::info!("{:?}", aws_tool);
        assert!(aws_tool.auto_allow_readonly);
        assert_eq!(2, aws_tool.allowed_services.len());
        assert_eq!(1, aws_tool.denied_services.len());
        assert!(aws_tool.allowed_services.contains("ec2"));
        assert!(aws_tool.allowed_services.contains("s3"));
        assert!(aws_tool.denied_services.contains("iam"));

        assert!(kiro_agent.tool_aliases.is_empty());

        let content = fs
            .read_to_string(
                PathBuf::from(".kiro")
                    .join("agents")
                    .join("dependabot.json"),
            )
            .await?;
        let kiro_agent: Agent = serde_json::from_str(&content)?;
        assert_eq!("dependabot", kiro_agent.name);
        let exec_tool: ExecuteShellTool = kiro_agent.get_tool(ToolTarget::Shell);
        tracing::info!("{:?}", exec_tool);
        assert!(exec_tool.allowed_commands.contains("git commit .*"));
        assert!(exec_tool.allowed_commands.contains("git push .*"));
        assert!(!exec_tool.denied_commands.contains("git commit .*"));
        assert!(!exec_tool.denied_commands.contains("git push .*"));

        let fs_tool: WriteTool = kiro_agent.get_tool(ToolTarget::Write);
        tracing::info!("{:?}", fs_tool);
        assert!(fs_tool.allowed_paths.contains(".*Cargo.toml.*"));
        assert!(!fs_tool.denied_paths.contains(".*Cargo.toml.*"));

        tracing::info!("{:?}", kiro_agent.hooks);
        assert_eq!(2, kiro_agent.hooks.len());
        assert!(kiro_agent.hooks.contains_key(&HookTrigger::AgentSpawn));
        assert_eq!(
            2,
            kiro_agent
                .hooks
                .get(&HookTrigger::AgentSpawn)
                .unwrap()
                .len()
        );
        Ok(())
    }
}
