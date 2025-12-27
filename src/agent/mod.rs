mod custom_tool;
pub mod hook;
mod mcp_config;
pub mod tools;
mod wrapper_types;
pub const DEFAULT_AGENT_RESOURCES: &[&str] = &["file://README.md", "file://AGENTS.md"];
pub const DEFAULT_APPROVE: [&str; 0] = [];
use {
    super::agent::hook::{Hook, HookTrigger},
    crate::{Result, kdl::KdlAgent},
    color_eyre::eyre::eyre,
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet},
        fmt::Display,
    },
};
pub use {
    custom_tool::{CustomToolConfig, OAuthConfig, TransportType, tool_default_timeout},
    mcp_config::McpServerConfig,
    tools::*,
    wrapper_types::OriginalToolName,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Agent {
    /// Name of the agent
    #[serde(default)]
    pub name: String,
    /// This field is not model facing and is mostly here for users to discern
    /// between agents
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The intention for this field is to provide high level context to the
    /// agent. This should be seen as the same category of context as a system
    /// prompt.
    #[serde(default)]
    pub prompt: Option<String>,
    /// Configuration for Model Context Protocol (MCP) servers
    #[serde(default)]
    pub mcp_servers: McpServerConfig,
    /// List of tools the agent can see. Use \"@{MCP_SERVER_NAME}/tool_name\" to
    /// specify tools from mcp servers. To include all tools from a server,
    /// use \"@{MCP_SERVER_NAME}\"
    #[serde(default)]
    pub tools: HashSet<String>,
    /// Tool aliases for remapping tool names
    #[serde(default)]
    pub tool_aliases: HashMap<OriginalToolName, String>,
    /// List of tools the agent is explicitly allowed to use
    #[serde(default)]
    pub allowed_tools: HashSet<String>,
    /// Files to include in the agent's context
    #[serde(default)]
    pub resources: HashSet<String>,
    /// Commands to run when a chat session is created
    #[serde(default)]
    pub hooks: HashMap<HookTrigger, Vec<Hook>>,
    /// Settings for specific tools. These are mostly for native tools. The
    /// actual schema differs by tools and is documented in detail in our
    /// documentation
    #[serde(default)]
    pub tools_settings: HashMap<String, serde_json::Value>,
    /// The model ID to use for this agent. If not specified, uses the default
    /// model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, rename = "includeMcpJson")]
    pub include_mcp_json: bool,
}

impl Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Agent {
    pub fn validate(&self) -> Result<()> {
        let schema: serde_json::Value = serde_json::from_str(crate::schema::SCHEMA)?;
        let validator = jsonschema::validator_for(&schema)?;
        let instance = serde_json::to_value(self)?;

        if let Err(e) = validator.validate(&instance) {
            return Err(eyre!(
                "Validation error: {}\n{}",
                e,
                serde_json::to_string(&instance)?
            ));
        }
        Ok(())
    }
}

impl TryFrom<&KdlAgent> for Agent {
    type Error = color_eyre::Report;

    fn try_from(value: &KdlAgent) -> std::result::Result<Self, Self::Error> {
        let native_tools = &value.native_tool;
        let mut tools_settings = HashMap::new();

        let tool: AwsTool = native_tools.into();
        if tool != AwsTool::default() {
            tools_settings.insert(
                ToolTarget::Aws.to_string(),
                serde_json::to_value(&tool).unwrap(),
            );
        }
        let tool: ReadTool = native_tools.into();
        if tool != ReadTool::default() {
            tools_settings.insert(
                ToolTarget::Read.to_string(),
                serde_json::to_value(&tool).unwrap(),
            );
        }
        let tool: WriteTool = native_tools.into();
        if tool != WriteTool::default() {
            tools_settings.insert(
                ToolTarget::Write.to_string(),
                serde_json::to_value(&tool).unwrap(),
            );
        }
        let tool: ExecuteShellTool = native_tools.into();
        if tool != ExecuteShellTool::default() {
            tools_settings.insert(
                ToolTarget::Shell.to_string(),
                serde_json::to_value(&tool).unwrap(),
            );
        }
        let default_agent = Self::default();
        let tools = value.tools().clone();
        let allowed_tools = value.allowed_tools().clone();
        let resources: HashSet<String> = value.resources().map(|s| s.to_string()).collect();

        // Extra tool settings override native tools
        let extra_tool_settings = value.extra_tool_settings()?;
        tools_settings.extend(extra_tool_settings);

        Ok(Self {
            name: value.name.clone(),
            description: value.description.clone(),
            prompt: value.prompt.clone(),
            mcp_servers: McpServerConfig {
                mcp_servers: value.mcp_servers(),
            },
            tools: if tools.is_empty() {
                default_agent.tools
            } else {
                tools
            },
            tool_aliases: value.tool_aliases(),
            allowed_tools: if allowed_tools.is_empty() {
                default_agent.allowed_tools
            } else {
                allowed_tools
            },
            resources: if resources.is_empty() {
                default_agent.resources
            } else {
                resources
            },
            hooks: value.hooks(),
            tools_settings,
            model: value.model.clone(),
            include_mcp_json: value.include_mcp_json(),
        })
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            name: "kiro_default".to_string(),
            description: Some("Default agent".to_string()),
            tools: {
                let mut set = HashSet::new();
                set.insert("*".to_string());
                set
            },
            prompt: Default::default(),
            mcp_servers: Default::default(),
            tool_aliases: Default::default(),
            allowed_tools: {
                let mut set = HashSet::<String>::new();
                let default_approve = DEFAULT_APPROVE.iter().copied().map(str::to_string);
                set.extend(default_approve);
                set
            },
            resources: {
                let mut resources = HashSet::new();
                resources.extend(DEFAULT_AGENT_RESOURCES.iter().map(|&s| s.into()));
                //                resources.insert(format!("file://{}", RULES_PATTERN).into());
                resources
            },
            hooks: Default::default(),
            tools_settings: Default::default(),
            include_mcp_json: true,
            model: None,
        }
    }
}
