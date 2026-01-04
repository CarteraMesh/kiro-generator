use {
    super::{
        GenericItem,
        agent::*,
        hook::HookDoc,
        mcp::CustomToolConfigDoc,
        native::NativeToolsDoc,
    },
    crate::Fs,
    facet::Facet,
    facet_kdl as kdl,
    miette::{Context, IntoDiagnostic},
    std::path::Path,
};

#[derive(Facet, Copy, Default, Clone, Debug, PartialEq, Eq)]
#[facet(default)]
pub(super) struct BoolDoc {
    #[facet(kdl::argument)]
    pub value: bool,
}
#[derive(Facet, Clone, Default)]
#[facet(deny_unknown_fields, rename_all = "kebab-case", default)]
pub struct KdlAgentFileDoc {
    #[facet(kdl::child, default)]
    pub(super) description: Option<GenericItem>,
    #[facet(kdl::children, default)]
    pub(super) inherits: Vec<GenericItem>,
    #[facet(kdl::child, default)]
    pub(super) prompt: Option<GenericItem>,
    #[facet(kdl::children, default)]
    pub(super) resources: Vec<GenericItem>,

    #[facet(kdl::child, default)]
    pub(super) include_mcp_json: Option<BoolDoc>,

    #[facet(kdl::children, default)]
    pub(super) tools: Vec<GenericItem>,

    #[facet(kdl::children, default)]
    pub(super) allowed_tools: Vec<GenericItem>,

    #[facet(kdl::child, default)]
    pub(super) model: Option<GenericItem>,

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

        let content = fs
            .read_to_string_sync(&path)
            .into_diagnostic()
            .wrap_err(format!("unable to read {}", path.as_ref().display()))?;
        let agent: KdlAgentFileDoc = kdl::from_str(&content).into_diagnostic().map_err(|e| {
            crate::format_err!("failed {} error:'{e}'\n{content}", path.as_ref().display())
        })?;
        Ok(Some(Self::from_file_source(name, agent)))
    }

    pub fn from_file_source(name: impl AsRef<str>, file_source: KdlAgentFileDoc) -> Self {
        Self {
            name: name.as_ref().to_string(),
            description: file_source.description,
            template: None,
            inherits: file_source.inherits,
            prompt: file_source.prompt,
            resources: file_source.resources,
            include_mcp_json: Some(file_source.include_mcp_json.unwrap_or_default().value),
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
