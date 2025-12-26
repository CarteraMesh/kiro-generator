use {
    super::{hook::HookPart, mcp::CustomToolConfigKdl},
    crate::{
        agent::{
            CustomToolConfig,
            OriginalToolName,
            hook::{Hook, HookTrigger},
        },
        kdl::native::{AwsTool, ExecuteShellTool, NativeTools, ReadTool, WriteTool},
    },
    knuffel::Decode,
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
        hash::Hash,
    },
};

#[derive(Decode, Clone, Default, Debug)]
pub(super) struct Inherits {
    #[knuffel(arguments, default)]
    pub parents: HashSet<String>,
}

#[derive(Decode, Clone, Default, Debug)]
pub(super) struct Tools {
    #[knuffel(arguments, default)]
    pub tools: HashSet<String>,
}

#[derive(Decode, Clone, Default, Debug)]
pub(super) struct AllowedTools {
    #[knuffel(arguments, default)]
    pub allowed: HashSet<String>,
}

#[derive(Decode, Clone, Default, Debug)]
pub(super) struct Resource {
    #[knuffel(argument)]
    pub location: String,
}

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        self.location.eq(&other.location)
    }
}

impl Hash for Resource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.location.hash(state);
    }
}
impl Eq for Resource {}

#[derive(Decode, Clone, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ToolAliasKdl {
    #[knuffel(argument)]
    from: String,
    #[knuffel(argument)]
    to: String,
}

#[derive(Decode, Clone, Default)]
pub struct KdlAgent {
    /// Name of the agent
    #[knuffel(argument)]
    pub name: String,
    /// Do not generate JSON Kiro agent, this is a "template" agent
    #[knuffel(property, default)]
    pub template: Option<bool>,
    #[knuffel(child, unwrap(argument))]
    pub description: Option<String>,
    #[knuffel(child, default)]
    pub(super) inherits: Inherits,
    /// The intention for this field is to provide high level context to the
    /// agent. This should be seen as the same category of context as a system
    /// prompt.
    #[knuffel(child, unwrap(argument))]
    pub prompt: Option<String>,
    /// Files to include in the agent's context
    #[knuffel(children(name = "resource"))]
    pub(super) resources: HashSet<Resource>,
    #[knuffel(child, default, unwrap(argument))]
    pub include_mcp_json: Option<bool>,
    /// List of tools the agent can see. Use \"@{MCP_SERVER_NAME}/tool_name\" to
    /// specify tools from mcp servers. To include all tools from a server,
    /// use \"@{MCP_SERVER_NAME}\"
    #[knuffel(child, default)]
    pub(super) tools: Tools,
    /// List of tools the agent is explicitly allowed to use
    #[knuffel(child, default)]
    pub(super) allowed_tools: AllowedTools,
    /// The model ID to use for this agent. If not specified, uses the default
    /// model.
    #[knuffel(child, unwrap(argument))]
    pub model: Option<String>,
    /// Commands to run when a chat session is created
    #[knuffel(child)]
    pub(super) hook: Option<HookPart>,
    #[knuffel(children(name = "mcp"), default)]
    pub(super) mcp: Vec<CustomToolConfigKdl>,
    #[knuffel(children(name = "alias"), default)]
    pub(super) tool_aliases: HashSet<ToolAliasKdl>,
    /// Tools builtin to kiro
    #[knuffel(child, default)]
    pub native_tool: NativeTools,
}

impl Debug for KdlAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for KdlAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl KdlAgent {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn is_template(&self) -> bool {
        self.template.is_some_and(|f| f)
    }

    pub fn include_mcp_json(&self) -> bool {
        self.include_mcp_json.is_some_and(|f| f)
    }

    pub fn get_tool_aws(&self) -> &AwsTool {
        &self.native_tool.aws
    }

    pub fn get_tool_read(&self) -> &ReadTool {
        &self.native_tool.read
    }

    pub fn get_tool_write(&self) -> &WriteTool {
        &self.native_tool.write
    }

    pub fn get_tool_shell(&self) -> &ExecuteShellTool {
        &self.native_tool.shell
    }

    pub fn tool_aliases(&self) -> HashMap<OriginalToolName, String> {
        self.tool_aliases
            .iter()
            .map(|m| (OriginalToolName(m.from.clone()), m.to.clone()))
            .collect()
    }

    pub fn hooks(&self) -> HashMap<HookTrigger, Vec<Hook>> {
        match &self.hook {
            None => HashMap::new(),
            Some(h) => h.triggers(),
        }
    }

    pub fn allowed_tools(&self) -> &HashSet<String> {
        &self.allowed_tools.allowed
    }

    pub fn tools(&self) -> &HashSet<String> {
        &self.tools.tools
    }

    pub fn inherits(&self) -> &HashSet<String> {
        &self.inherits.parents
    }

    pub fn resources(&self) -> impl Iterator<Item = &str> {
        self.resources.iter().map(|r| r.location.as_str())
    }

    pub fn mcp_servers(&self) -> HashMap<String, CustomToolConfig> {
        self.mcp
            .iter()
            .map(|m| (m.name.clone(), m.into()))
            .collect()
    }
}
