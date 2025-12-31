use {
    super::{
        hook::{HookDoc, HookPart},
        mcp::CustomToolConfigDoc,
        native::{NativeTools, NativeToolsDoc},
    },
    crate::agent::{CustomToolConfig, OriginalToolName},
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
    #[allow(dead_code)]
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

#[derive(Clone, Default)]
pub struct KdlAgent {
    pub name: String,
    pub template: Option<bool>,
    pub description: Option<String>,
    pub inherits: HashSet<String>,
    pub prompt: Option<String>,
    pub resources: HashSet<String>,
    pub include_mcp_json: Option<bool>,
    pub tools: HashSet<String>,
    pub allowed_tools: HashSet<String>,
    pub model: Option<String>,
    pub hook: HookPart,
    pub mcp: HashMap<String, CustomToolConfig>,
    pub alias: HashMap<OriginalToolName, String>,
    pub native_tool: NativeTools,
    pub tool_setting: Vec<ToolSetting>,
}

#[derive(Facet, Clone, Default)]
#[facet(rename = "kabel-case", default)]
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
    #[facet(kdl::property, default)]
    pub include_mcp_json: Option<bool>,
    #[facet(kdl::child, default)]
    pub(super) tools: Option<Tools>,
    #[facet(kdl::child, default)]
    pub(super) allowed_tools: Option<AllowedTools>,
    #[facet(kdl::child, default)]
    pub model: Option<Model>,
    #[facet(kdl::child, default)]
    pub(super) hook: Option<HookDoc>,
    #[facet(kdl::children, default)]
    pub(super) mcp: Vec<CustomToolConfigDoc>,
    #[facet(kdl::children, default)]
    pub(super) alias: Vec<ToolAliasKdl>,
    #[facet(kdl::child, default)]
    pub native_tool: NativeToolsDoc,
    #[facet(kdl::children, default)]
    pub(super) tool_setting: Vec<ToolSetting>,
}

#[derive(Facet, Clone, Default, Debug)]
pub struct Description {
    #[facet(kdl::argument)]
    value: String,
}

#[derive(Facet, Clone, Default, Debug)]
pub struct Prompt {
    #[facet(kdl::argument)]
    value: String,
}

#[derive(Facet, Clone, Debug)]
pub struct Model {
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

impl From<KdlAgentDoc> for KdlAgent {
    fn from(value: KdlAgentDoc) -> Self {
        Self {
            name: value.name.clone(),
            description: value.description.as_ref().map(|f| f.value.clone()),
            prompt: value.prompt.as_ref().map(|f| f.value.clone()),
            alias: value.tool_aliases(),
            allowed_tools: value.allowed_tools(),
            inherits: value.inherits(),
            template: value.template,
            include_mcp_json: value.include_mcp_json,
            hook: value.hooks(),
            resources: value.resources(),
            model: value.model.as_ref().map(|f| f.value.clone()),
            mcp: value.mcp_servers(),
            tools: value.tools(),
            tool_setting: Default::default(), // TODO use facet::Value
            native_tool: value.native_tool.into(),
        }
    }
}

impl KdlAgent {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn is_template(&self) -> bool {
        self.template.is_some_and(|f| f)
    }
}
impl KdlAgentDoc {
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

    pub fn tool_aliases(&self) -> HashMap<OriginalToolName, String> {
        self.alias
            .iter()
            .map(|m| (OriginalToolName(m.from.clone()), m.to.clone()))
            .collect()
    }

    pub fn hooks(&self) -> HookPart {
        HookPart::from(self.hook.clone().unwrap_or_default())
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

    pub fn resources(&self) -> HashSet<String> {
        HashSet::from_iter(self.resources.iter().map(|r| r.location.clone()))
    }

    pub fn mcp_servers(&self) -> HashMap<String, CustomToolConfig> {
        self.mcp
            .iter()
            .map(|m| (m.name.clone(), m.into()))
            .collect()
    }

    pub fn extra_tool_settings(&self) -> crate::Result<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
        // for setting in &self.tool_setting {
        //     let (name, value) = setting.to_value()?;
        //     if result.contains_key(&name) {
        //         return Err(color_eyre::eyre::eyre!(
        //             "[{self}] - Duplicate tool-setting '{}' found. Each
        // tool-setting name must be \              unique.",
        //             name
        //         ));
        //     }
        //     result.insert(name, value);
        // }
    }
}
