use {
    crate::agent::{
        AwsTool as KiroAwsTool,
        ExecuteShellTool as KiroShellTool,
        ReadTool as KiroReadTool,
        WriteTool as KiroWriteTool,
    },
    knuffel::Decode,
    std::collections::HashSet,
};

#[derive(Decode, Debug, Clone, Default)]
pub(super) struct GenericList {
    #[knuffel(arguments)]
    pub list: Vec<String>,
}

#[derive(Decode, Debug, Clone, Default)]
pub(super) struct Force {
    #[knuffel(argument)]
    pub path: String,
}

#[derive(Decode, Clone, Debug, Default)]
pub struct WriteTool {
    #[knuffel(child)]
    pub allow: Option<GenericList>,
    #[knuffel(child)]
    pub deny: Option<GenericList>,
    #[knuffel(children(name = "force"))]
    pub force: Vec<Force>,
}

#[derive(Decode, Clone, Debug, Default)]
pub struct ReadTool {
    #[knuffel(child)]
    pub allow: Option<GenericList>,
    #[knuffel(child)]
    pub deny: Option<GenericList>,
    #[knuffel(children(name = "force"))]
    pub force: Vec<Force>,
}

#[derive(Decode, Debug, Default, Clone)]
pub struct AwsTool {
    #[knuffel(child)]
    pub allow: Option<GenericList>,
    #[knuffel(child)]
    pub deny: Option<GenericList>,
}

#[derive(Decode, Debug, Clone, Default)]
pub struct ExecuteShellTool {
    #[knuffel(child)]
    pub allow: Option<GenericList>,
    #[knuffel(child)]
    pub deny: Option<GenericList>,
    #[knuffel(property)]
    pub deny_by_default: bool,
    #[knuffel(argument, default)]
    pub disable_auto_readonly: bool,
    #[knuffel(children(name = "force"))]
    pub force: Vec<Force>,
}

#[derive(Decode, Default, Clone, Debug)]
pub struct NativeTools {
    #[knuffel(child)]
    shell: Option<ExecuteShellTool>,
    #[knuffel(child)]
    aws: Option<AwsTool>,
    #[knuffel(child)]
    read: Option<ReadTool>,
    #[knuffel(child)]
    write: Option<WriteTool>,
}

impl From<&NativeTools> for KiroAwsTool {
    fn from(value: &NativeTools) -> Self {
        match &value.aws {
            None => KiroAwsTool::default(),
            Some(t) => KiroAwsTool {
                allowed_services: t.allow.clone().map_or(HashSet::default(), |f| {
                    HashSet::from_iter(f.list.iter().cloned())
                }),
                denied_services: t.deny.clone().map_or(HashSet::default(), |f| {
                    HashSet::from_iter(f.list.iter().cloned())
                }),
                auto_allow_readonly: true,
            },
        }
    }
}

impl From<&NativeTools> for KiroWriteTool {
    fn from(value: &NativeTools) -> Self {
        match &value.write {
            None => KiroWriteTool::default(),
            Some(t) => KiroWriteTool {
                allowed_paths: t.allow.clone().map_or(HashSet::default(), |f| {
                    HashSet::from_iter(f.list.iter().cloned())
                }),
                denied_paths: t.deny.clone().map_or(HashSet::default(), |f| {
                    HashSet::from_iter(f.list.iter().cloned())
                }),
            },
        }
    }
}

impl From<&NativeTools> for KiroReadTool {
    fn from(value: &NativeTools) -> Self {
        match &value.read {
            None => KiroReadTool::default(),
            Some(t) => KiroReadTool {
                allowed_paths: t.allow.clone().map_or(HashSet::default(), |f| {
                    HashSet::from_iter(f.list.iter().cloned())
                }),
                denied_paths: t.deny.clone().map_or(HashSet::default(), |f| {
                    HashSet::from_iter(f.list.iter().cloned())
                }),
            },
        }
    }
}

impl From<&NativeTools> for KiroShellTool {
    fn from(value: &NativeTools) -> Self {
        match &value.shell {
            None => KiroShellTool::default(),
            Some(t) => KiroShellTool {
                allowed_commands: t.allow.clone().map_or(HashSet::default(), |f| {
                    HashSet::from_iter(f.list.iter().cloned())
                }),
                denied_commands: t.deny.clone().map_or(HashSet::default(), |f| {
                    HashSet::from_iter(f.list.iter().cloned())
                }),
                deny_by_default: t.deny_by_default,
                auto_allow_readonly: !t.disable_auto_readonly,
            },
        }
    }
}
