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
pub struct GenericList {
    #[knuffel(arguments)]
    pub list: HashSet<String>,
}

#[derive(Decode, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Force {
    #[knuffel(argument)]
    pub path: String,
}

impl From<&str> for Force {
    fn from(value: &str) -> Self {
        Self {
            path: value.to_string(),
        }
    }
}

#[derive(Decode, Clone, Debug, Default)]
pub struct WriteTool {
    #[knuffel(child, default)]
    pub allow: GenericList,
    #[knuffel(child, default)]
    pub deny: GenericList,
    #[knuffel(children(name = "force"))]
    pub force: HashSet<Force>,
}

impl WriteTool {
    fn merge(mut self, other: Self) -> Self {
        self.allow.list.extend(other.allow.list);
        self.deny.list.extend(other.deny.list);
        self.force.extend(other.force);
        self
    }
}

#[derive(Decode, Clone, Debug, Default)]
pub struct ReadTool {
    #[knuffel(child, default)]
    pub allow: GenericList,
    #[knuffel(child, default)]
    pub deny: GenericList,
    #[knuffel(children(name = "force"))]
    pub force: HashSet<Force>,
}

impl ReadTool {
    fn merge(mut self, other: Self) -> Self {
        self.allow.list.extend(other.allow.list);
        self.deny.list.extend(other.deny.list);
        self.force.extend(other.force);
        self
    }
}

#[derive(Decode, Debug, Default, Clone)]
pub struct AwsTool {
    #[knuffel(child, default)]
    pub allow: GenericList,
    #[knuffel(child, default)]
    pub deny: GenericList,
}

impl AwsTool {
    fn merge(mut self, other: Self) -> Self {
        self.allow.list.extend(other.allow.list);
        self.deny.list.extend(other.deny.list);
        self
    }
}

#[derive(Decode, Debug, Clone, Default)]
pub struct ExecuteShellTool {
    #[knuffel(child, default)]
    pub allow: GenericList,
    #[knuffel(child, default)]
    pub deny: GenericList,
    #[knuffel(property, default)]
    pub deny_by_default: bool,
    #[knuffel(argument, default)]
    pub disable_auto_readonly: bool,
    #[knuffel(children(name = "force"))]
    pub force: HashSet<Force>,
}

impl ExecuteShellTool {
    fn merge(mut self, other: Self) -> Self {
        self.allow.list.extend(other.allow.list);
        self.deny.list.extend(other.deny.list);
        self.deny_by_default |= other.deny_by_default;
        self.disable_auto_readonly |= other.disable_auto_readonly;
        self.force.extend(other.force);
        self
    }
}

#[derive(Decode, Default, Clone, Debug)]
pub struct NativeTools {
    #[knuffel(child, default)]
    pub shell: ExecuteShellTool,
    #[knuffel(child, default)]
    pub aws: AwsTool,
    #[knuffel(child, default)]
    pub read: ReadTool,
    #[knuffel(child, default)]
    pub write: WriteTool,
}

impl NativeTools {
    pub fn merge(mut self, other: Self) -> Self {
        self.shell = self.shell.merge(other.shell);
        self.aws = self.aws.merge(other.aws);
        self.read = self.read.merge(other.read);
        self.write = self.write.merge(other.write);
        self
    }
}

impl From<&NativeTools> for KiroAwsTool {
    fn from(value: &NativeTools) -> Self {
        KiroAwsTool {
            allowed_services: HashSet::from_iter(value.aws.allow.list.iter().cloned()),
            denied_services: HashSet::from_iter(value.aws.deny.list.iter().cloned()),
            auto_allow_readonly: true,
        }
    }
}

impl From<&NativeTools> for KiroWriteTool {
    fn from(value: &NativeTools) -> Self {
        KiroWriteTool {
            allowed_paths: HashSet::from_iter(value.write.allow.list.iter().cloned()),
            denied_paths: HashSet::from_iter(value.write.deny.list.iter().cloned()),
        }
    }
}

impl From<&NativeTools> for KiroReadTool {
    fn from(value: &NativeTools) -> Self {
        KiroReadTool {
            allowed_paths: HashSet::from_iter(value.read.allow.list.iter().cloned()),
            denied_paths: HashSet::from_iter(value.read.deny.list.iter().cloned()),
        }
    }
}

impl From<&NativeTools> for KiroShellTool {
    fn from(value: &NativeTools) -> Self {
        KiroShellTool {
            allowed_commands: HashSet::from_iter(value.shell.allow.list.iter().cloned()),
            denied_commands: HashSet::from_iter(value.shell.deny.list.iter().cloned()),
            deny_by_default: value.shell.deny_by_default,
            auto_allow_readonly: !value.shell.disable_auto_readonly,
        }
    }
}
