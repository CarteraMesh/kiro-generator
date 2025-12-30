use {
    super::{
        hook::HookDoc,
        mcp::CustomToolConfigKdl,
        native::{AwsTool, ExecuteShellTool, NativeTools, NativeToolsDoc, ReadTool, WriteTool},
    },
    crate::agent::{
        CustomToolConfig, OriginalToolName,
        hook::{Hook, HookTrigger},
    },
    color_eyre::eyre::WrapErr,
    facet::Facet,
    facet_kdl as kdl,
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
        hash::Hash,
    },
};

#[derive(Facet, Clone, Default, Debug)]
pub(super) struct Inherits {
    #[facet(kdl::arguments)]
    pub parents: Vec<String>,
}

#[derive(Facet, Clone, Default, Debug)]
pub(super) struct Tools {
    #[facet(kdl::arguments)]
    pub tools: Vec<String>,
}

#[derive(Facet, Clone, Default, Debug)]
pub(super) struct AllowedTools {
    #[facet(kdl::arguments)]
    pub allowed: Vec<String>,
}

#[derive(Facet, Clone, Default, Debug)]
pub(super) struct Resource {
    #[facet(kdl::argument)]
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

#[derive(Facet, Clone, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ToolAliasKdl {
    #[facet(kdl::argument)]
    from: String,
    #[facet(kdl::argument)]
    to: String,
}

#[derive(Facet, Clone, Debug)]
pub struct ToolSetting {
    #[facet(kdl::argument)]
    name: String,
    #[facet(kdl::child)]
    json: Json,
}

#[derive(Facet, Clone, Debug)]
struct Json {
    #[facet(kdl::argument)]
    value: String,
}

impl ToolSetting {
    fn to_value(&self) -> crate::Result<(String, serde_json::Value)> {
        let v: serde_json::Value = serde_json::from_str(&self.json.value)
            .wrap_err_with(|| format!("Failed to parse JSON for tool-setting '{}'", self.name))?;

        if !v.is_object() {
            return Err(color_eyre::eyre::eyre!(
                "tool-setting '{}' must be a JSON object, got: {}",
                self.name,
                v
            ));
        }

        Ok((self.name.clone(), v))
    }
}

#[derive(Facet, Clone, Default)]
pub struct KdlAgentDoc {
    #[facet(kdl::argument)]
    pub name: String,
    #[facet(kdl::property)]
    pub template: Option<bool>,
    #[facet(kdl::child, default)]
    pub description: Option<Description>,
    #[facet(kdl::child, default)]
    pub(super) inherits: Inherits,
    #[facet(kdl::child, default)]
    pub prompt: Option<Prompt>,
    #[facet(kdl::children, default)]
    pub(super) resources: Vec<Resource>,
    #[facet(kdl::child, default)]
    pub include_mcp_json: Option<IncludeMcpJson>,
    #[facet(kdl::child, default)]
    pub(super) tools: Option<Tools>,
    #[facet(kdl::child, default)]
    pub(super) allowed_tools: Option<AllowedTools>,
    #[facet(kdl::child, default)]
    pub model: Option<Model>,
    #[facet(kdl::child, default)]
    pub(super) hook: Option<HookDoc>,
    #[facet(kdl::children, default)]
    pub(super) mcp: Vec<CustomToolConfigKdl>,
    #[facet(kdl::children, default)]
    pub(super) alias: Vec<ToolAliasKdl>,
    #[facet(kdl::child, default)]
    pub native_tool: Option<NativeToolsDoc>,
    #[facet(kdl::children, default)]
    pub(super) tool_setting: Vec<ToolSetting>,
}

#[derive(Facet, Clone, Default, Debug)]
pub struct Description {
    #[facet(kdl::argument)]
    value: String,
}

#[derive(Facet, Clone, Default, Debug)]
struct Prompt {
    #[facet(kdl::argument)]
    value: String,
}

#[derive(Facet, Clone, Debug)]
struct IncludeMcpJson {
    #[facet(kdl::argument)]
    value: bool,
}

#[derive(Facet, Clone, Debug)]
struct Model {
    #[facet(kdl::argument)]
    value: String,
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
    pub fn prompt(&self) -> String {
        self.prompt.clone().unwrap_or_default().value
    }

    pub fn description(&self) -> String {
        self.description.clone().unwrap_or_default().value
    }

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
        self.include_mcp_json
            .as_ref()
            .map(|i| i.value)
            .unwrap_or(false)
    }

    pub fn get_tool_aws(&self) -> AwsTool {
        self.native_tool
            .as_ref()
            .and_then(|n| n.aws.clone())
            .unwrap_or_default()
    }

    pub fn get_tool_read(&self) -> ReadTool {
        self.native_tool
            .as_ref()
            .and_then(|n| n.read.clone())
            .unwrap_or_default()
    }

    pub fn get_tool_write(&self) -> WriteTool {
        self.native_tool
            .as_ref()
            .and_then(|n| n.write.clone())
            .unwrap_or_default()
    }

    pub fn get_tool_shell(&self) -> ExecuteShellTool {
        self.native_tool
            .as_ref()
            .and_then(|n| n.shell.clone())
            .unwrap_or_default()
    }

    pub fn tool_aliases(&self) -> HashMap<OriginalToolName, String> {
        self.alias
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

    pub fn allowed_tools(&self) -> HashSet<String> {
        self.allowed_tools
            .as_ref()
            .map(|a| HashSet::from_iter(a.allowed.clone()))
            .unwrap_or_default()
    }

    pub fn tools(&self) -> HashSet<String> {
        self.tools
            .as_ref()
            .map(|t| HashSet::from_iter(t.tools.clone()))
            .unwrap_or_default()
    }

    pub fn inherits(&self) -> HashSet<String> {
        HashSet::from_iter(self.inherits.parents.clone())
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

    pub fn extra_tool_settings(&self) -> crate::Result<HashMap<String, serde_json::Value>> {
        let mut result = HashMap::new();
        for setting in &self.tool_setting {
            let (name, value) = setting.to_value()?;
            if result.contains_key(&name) {
                return Err(color_eyre::eyre::eyre!(
                    "[{self}] - Duplicate tool-setting '{}' found. Each tool-setting name must be \
                     unique.",
                    name
                ));
            }
            result.insert(name, value);
        }
        Ok(result)
    }
}
