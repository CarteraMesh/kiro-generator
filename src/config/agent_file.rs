use {
    super::{agent::*, hook::HookDoc, mcp::CustomToolConfigDoc, native::NativeToolsDoc},
    crate::Fs,
    color_eyre::eyre::eyre,
    facet::Facet,
    facet_kdl as kdl,
    miette::IntoDiagnostic,
    std::path::Path,
};

#[derive(Facet, Clone, Default)]
#[facet(rename = "kabel-case", default)]
pub struct KdlAgentFileDoc {
    #[facet(kdl::child, default)]
    pub description: Option<Description>,
    #[facet(kdl::child, default)]
    pub(super) inherits: Inherits,
    #[facet(kdl::child, default)]
    pub(super) prompt: Option<Prompt>,
    #[facet(kdl::children, default)]
    pub(super) resources: Vec<Resource>,
    #[facet(kdl::property, default)]
    pub include_mcp_json: Option<bool>,
    #[facet(kdl::child, default)]
    pub(super) tools: Option<Tools>,
    #[facet(kdl::child, default)]
    pub(super) allowed_tools: Option<AllowedTools>,
    #[facet(kdl::child, default)]
    pub(super) model: Option<Model>,
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

impl KdlAgentDoc {
    pub fn from_path(
        fs: &Fs,
        name: impl AsRef<str>,
        path: impl AsRef<Path>,
    ) -> crate::Result<Option<Self>> {
        if !fs.exists(&path) {
            return Ok(None);
        }

        let content = fs.read_to_string_sync(&path)?;
        let agent: KdlAgentFileDoc = kdl::from_str(&content)
            .into_diagnostic()
            .map_err(|e| eyre!("failed {} {e}", path.as_ref().display()))?;
        Ok(Some(Self::from_file_source(name, agent)))
    }

    pub fn from_file_source(name: impl AsRef<str>, file_source: KdlAgentFileDoc) -> Self {
        Self {
            name: name.as_ref().to_string(),
            description: file_source.description,
            template: None,
            inherits: Inherits::default(),
            prompt: file_source.prompt,
            resources: file_source.resources,
            include_mcp_json: file_source.include_mcp_json,
            tools: file_source.tools,
            allowed_tools: file_source.allowed_tools,
            model: file_source.model,
            hook: file_source.hook,
            mcp: file_source.mcp,
            alias: file_source.alias,
            native_tool: file_source.native_tool,
            tool_setting: file_source.tool_setting,
        }
    }
}
