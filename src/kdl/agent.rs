use {
    super::{hook::HookPart, mcp::CustomToolConfigKdl},
    crate::{
        agent::{
            AwsTool as KiroAwsTool,
            CustomToolConfig,
            ExecuteShellTool as KiroShellTool,
            OriginalToolName,
            ReadTool as KiroReadTool,
            WriteTool as KiroWriteTool,
            hook::HookTrigger,
        },
        kdl::native::NativeTools,
    },
    knuffel::Decode,
    std::{
        collections::{HashMap, HashSet},
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

#[derive(Decode, Clone, Debug)]
pub struct KdlAgent {
    /// Name of the agent
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument))]
    pub description: Option<String>,
    #[knuffel(child, default)]
    pub skeleton: bool,
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
    #[knuffel(child, default)]
    pub include_mcp_json: bool,
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
    mcp: Vec<CustomToolConfigKdl>,
    #[knuffel(children(name = "alias"), default)]
    pub(super) tool_aliases: HashSet<ToolAliasKdl>,
    /// Tools builtin to kiro
    #[knuffel(child, default)]
    native: NativeTools,
}

impl KdlAgent {
    pub fn get_tool_aws(&self) -> KiroAwsTool {
        KiroAwsTool::from(&self.native)
    }

    pub fn get_tool_read(&self) -> KiroReadTool {
        KiroReadTool::from(&self.native)
    }

    pub fn get_tool_write(&self) -> KiroWriteTool {
        KiroWriteTool::from(&self.native)
    }

    pub fn get_tool_shell(&self) -> KiroShellTool {
        KiroShellTool::from(&self.native)
    }

    pub fn tool_aliases(&self) -> HashMap<OriginalToolName, String> {
        self.tool_aliases
            .clone()
            .into_iter()
            .map(|m| (OriginalToolName(m.from.clone()), m.to.clone()))
            .collect()
    }

    pub fn hook(&self, trigger: HookTrigger) -> Option<crate::agent::hook::Hook> {
        self.hook.as_ref().and_then(|h| h.get(trigger))
    }

    pub fn allowed_tools(&self) -> HashSet<String> {
        self.allowed_tools.allowed.clone()
    }

    pub fn tools(&self) -> HashSet<String> {
        self.tools.tools.clone()
    }

    pub fn inherits(&self) -> HashSet<String> {
        self.inherits.parents.clone()
    }

    pub fn resources(&self) -> Vec<String> {
        self.resources
            .clone()
            .into_iter()
            .map(|r| r.location)
            .collect()
    }

    pub fn mcp_servers(&self) -> HashMap<String, CustomToolConfig> {
        self.mcp
            .clone()
            .into_iter()
            .map(|m| (m.name.clone(), m.into()))
            .collect()
    }
}
