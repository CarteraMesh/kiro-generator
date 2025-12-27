use {
    super::{agent::*, hook::HookPart, mcp::CustomToolConfigKdl, native::NativeTools},
    crate::os::Fs,
    color_eyre::eyre::eyre,
    knuffel::{Decode, parse},
    std::{collections::HashSet, path::Path},
};

#[derive(Decode, Clone, Debug)]
pub struct KdlAgentFileSource {
    #[knuffel(child, unwrap(argument))]
    pub description: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub prompt: Option<String>,
    #[knuffel(children(name = "resource"))]
    pub(super) resources: HashSet<Resource>,
    #[knuffel(child, default, unwrap(argument))]
    pub include_mcp_json: Option<bool>,
    #[knuffel(child, default)]
    pub(super) tools: Tools,
    #[knuffel(child, default)]
    pub(super) allowed_tools: AllowedTools,
    #[knuffel(child, unwrap(argument))]
    pub model: Option<String>,
    #[knuffel(child)]
    pub(super) hook: Option<HookPart>,
    #[knuffel(children(name = "mcp"), default)]
    pub(super) mcp: Vec<CustomToolConfigKdl>,
    #[knuffel(children(name = "alias"), default)]
    pub(super) tool_aliases: HashSet<ToolAliasKdl>,
    #[knuffel(child, default)]
    pub(super) native_tool: NativeTools,
    #[knuffel(children(name = "tool-setting"), default)]
    pub(super) tool_settings: Vec<ToolSetting>,
}

impl KdlAgent {
    pub fn from_path(
        fs: &Fs,
        name: impl AsRef<str>,
        path: impl AsRef<Path>,
    ) -> crate::Result<Option<Self>> {
        if !fs.exists(&path) {
            return Ok(None);
        }

        let content = fs.read_to_string_sync(&path)?;
        let path_str = format!("{}", path.as_ref().display());
        let agent: KdlAgentFileSource = match parse(&path_str, &content) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{:?}", miette::Report::new(e));
                return Err(eyre!("failed to parse agent file"));
            }
        };
        Ok(Some(Self::from_file_source(name, agent)))
    }

    pub fn from_file_source(name: impl AsRef<str>, file_source: KdlAgentFileSource) -> Self {
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
            tool_aliases: file_source.tool_aliases,
            native_tool: file_source.native_tool,
            tool_settings: file_source.tool_settings,
        }
    }
}
