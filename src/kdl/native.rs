use {
    crate::agent::{
        AwsTool as KiroAwsTool,
        ExecuteShellTool as KiroShellTool,
        ReadTool as KiroReadTool,
        WriteTool as KiroWriteTool,
    },
    knuffel::Decode,
    std::{collections::HashSet, fmt::Display},
};

#[derive(Decode, Debug, Clone, Default, PartialEq, Eq)]
pub struct GenericList {
    #[knuffel(arguments)]
    pub list: HashSet<String>,
}

impl From<&'static str> for GenericList {
    fn from(value: &'static str) -> Self {
        Self {
            list: HashSet::from_iter(vec![value.to_string()]),
        }
    }
}

impl FromIterator<&'static str> for GenericList {
    fn from_iter<T: IntoIterator<Item = &'static str>>(iter: T) -> Self {
        Self {
            list: HashSet::from_iter(iter.into_iter().map(|f| f.to_string())),
        }
    }
}

#[derive(Decode, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Override {
    #[knuffel(argument)]
    pub path: String,
}

impl Display for Override {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl From<&str> for Override {
    fn from(value: &str) -> Self {
        Self {
            path: value.to_string(),
        }
    }
}

#[derive(Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct WriteTool {
    #[knuffel(child, default)]
    pub allow: GenericList,
    #[knuffel(child, default)]
    pub deny: GenericList,
    #[knuffel(children(name = "override"))]
    pub override_path: HashSet<Override>,
}

impl WriteTool {
    fn merge(mut self, other: Self) -> Self {
        self.allow.list.extend(other.allow.list);
        self.deny.list.extend(other.deny.list);
        self.override_path.extend(other.override_path);
        self
    }
}

#[derive(Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct ReadTool {
    #[knuffel(child, default)]
    pub allow: GenericList,
    #[knuffel(child, default)]
    pub deny: GenericList,
    #[knuffel(children(name = "override"))]
    pub override_path: HashSet<Override>,
}

impl ReadTool {
    fn merge(mut self, other: Self) -> Self {
        self.allow.list.extend(other.allow.list);
        self.deny.list.extend(other.deny.list);
        self.override_path.extend(other.override_path);
        self
    }
}

#[derive(Decode, Debug, Default, Clone, PartialEq, Eq)]
pub struct AwsTool {
    #[knuffel(property)]
    pub disable_auto_readonly: Option<bool>,
    #[knuffel(child, default)]
    pub allow: GenericList,
    #[knuffel(child, default)]
    pub deny: GenericList,
}

impl AwsTool {
    fn merge(mut self, other: Self) -> Self {
        self.disable_auto_readonly = self.disable_auto_readonly.or(other.disable_auto_readonly);
        self.allow.list.extend(other.allow.list);
        self.deny.list.extend(other.deny.list);
        self
    }
}

#[derive(Decode, Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecuteShellTool {
    #[knuffel(child, default)]
    pub allow: GenericList,
    #[knuffel(child, default)]
    pub deny: GenericList,
    #[knuffel(property)]
    pub deny_by_default: Option<bool>,
    #[knuffel(property)]
    pub disable_auto_readonly: Option<bool>,
    #[knuffel(children(name = "override"))]
    pub override_command: HashSet<Override>,
}

impl ExecuteShellTool {
    fn merge(mut self, other: Self) -> Self {
        self.allow.list.extend(other.allow.list);
        self.deny.list.extend(other.deny.list);
        self.deny_by_default = self.deny_by_default.or(other.deny_by_default);
        self.disable_auto_readonly = self.disable_auto_readonly.or(other.disable_auto_readonly);
        self.override_command.extend(other.override_command);
        self
    }
}

#[derive(Decode, Default, Clone, Debug, PartialEq, Eq)]
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
            auto_allow_readonly: match value.aws.disable_auto_readonly {
                None => true,
                Some(f) => !f,
            },
        }
    }
}

impl From<&NativeTools> for KiroWriteTool {
    fn from(value: &NativeTools) -> Self {
        let mut allow: HashSet<String> = HashSet::from_iter(value.write.allow.list.iter().cloned());
        let mut deny: HashSet<String> = HashSet::from_iter(value.write.deny.list.iter().cloned());
        if !value.write.override_path.is_empty() {
            tracing::trace!(
                "Override/Forcing write: {:?}",
                value.shell.override_command.iter().collect::<Vec<_>>()
            );
            for cmd in value.write.override_path.iter() {
                allow.insert(cmd.path.clone());
                if deny.remove(&cmd.path) {
                    tracing::trace!("Removed from deny: {cmd}");
                }
            }
        }
        KiroWriteTool {
            allowed_paths: allow,
            denied_paths: deny,
        }
    }
}

impl From<&NativeTools> for KiroReadTool {
    fn from(value: &NativeTools) -> Self {
        let mut allow: HashSet<String> = HashSet::from_iter(value.read.allow.list.iter().cloned());
        let mut deny: HashSet<String> = HashSet::from_iter(value.read.deny.list.iter().cloned());
        if !value.read.override_path.is_empty() {
            tracing::trace!(
                "Override/Forcing read: {:?}",
                value.shell.override_command.iter().collect::<Vec<_>>()
            );
            for cmd in value.read.override_path.iter() {
                allow.insert(cmd.path.clone());
                if deny.remove(&cmd.path) {
                    tracing::trace!("Removed from deny: {cmd}");
                }
            }
        }
        KiroReadTool {
            allowed_paths: allow,
            denied_paths: deny,
        }
    }
}

impl From<&NativeTools> for KiroShellTool {
    fn from(value: &NativeTools) -> Self {
        let mut allow: HashSet<String> = HashSet::from_iter(value.shell.allow.list.iter().cloned());
        let mut deny: HashSet<String> = HashSet::from_iter(value.shell.deny.list.iter().cloned());
        if !value.shell.override_command.is_empty() {
            tracing::trace!(
                "Override/Forcing commands: {:?}",
                value.shell.override_command.iter().collect::<Vec<_>>()
            );
            for cmd in value.shell.override_command.iter() {
                allow.insert(cmd.path.clone());
                if deny.remove(&cmd.path) {
                    tracing::trace!("Removed command from deny: {cmd}");
                }
            }
        }

        KiroShellTool {
            allowed_commands: allow,
            denied_commands: deny,
            deny_by_default: value.shell.deny_by_default.unwrap_or(false),
            auto_allow_readonly: !(value.shell.disable_auto_readonly.unwrap_or(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::Result};

    #[test_log::test]
    pub fn test_native_merge_empty() -> Result<()> {
        let child = NativeTools::default();
        let parent = NativeTools::default();
        let merged = child.merge(parent);

        assert_eq!(merged, NativeTools::default());

        Ok(())
    }

    #[test_log::test]
    pub fn test_native_merge_empty_child() -> Result<()> {
        let child = NativeTools::default();
        let mut parent = NativeTools::default();
        let aws = AwsTool {
            disable_auto_readonly: None,
            allow: vec!["ec2"].into_iter().collect(),
            deny: vec!["iam"].into_iter().collect(),
        };

        let shell = ExecuteShellTool {
            allow: vec!["ls .*"].into_iter().collect(),
            deny: vec!["git push"].into_iter().collect(),
            override_command: HashSet::from_iter(vec![Override::from("rm -rf /")]),
            deny_by_default: Some(true),
            disable_auto_readonly: Some(false),
        };

        let read = ReadTool {
            allow: vec!["ls .*"].into_iter().collect(),
            deny: vec!["git push"].into_iter().collect(),
            override_path: HashSet::from_iter(vec![Override::from("rm -rf /")]),
        };
        let write = WriteTool {
            allow: vec!["ls .*"].into_iter().collect(),
            deny: vec!["git push"].into_iter().collect(),
            override_path: HashSet::from_iter(vec![Override::from("rm -rf /")]),
        };
        parent.aws = aws.clone();
        parent.shell = shell.clone();
        parent.read = read.clone();
        parent.write = write.clone();
        let merged = child.merge(parent);
        assert_eq!(merged.aws, aws);
        assert_eq!(merged.shell, shell);
        assert_eq!(merged.read, read);
        assert_eq!(merged.write, write);
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_merge_child_parent() -> Result<()> {
        let mut child = NativeTools::default();
        let parent = NativeTools {
            aws: AwsTool {
                disable_auto_readonly: None,
                allow: vec!["ec2"].into_iter().collect(),
                deny: vec!["iam"].into_iter().collect(),
            },
            shell: ExecuteShellTool {
                allow: vec!["ls .*"].into_iter().collect(),
                deny: vec!["git push"].into_iter().collect(),
                override_command: HashSet::from_iter(vec![Override::from("rm -rf /")]),
                deny_by_default: Some(true),
                disable_auto_readonly: Some(false),
            },

            read: ReadTool {
                allow: vec!["ls .*"].into_iter().collect(),
                deny: vec!["git push"].into_iter().collect(),
                override_path: HashSet::from_iter(vec![Override::from("rm -rf /")]),
            },
            write: WriteTool {
                allow: vec!["ls .*"].into_iter().collect(),
                deny: vec!["git push"].into_iter().collect(),
                override_path: HashSet::from_iter(vec![Override::from("rm -rf /")]),
            },
        };

        child.aws = AwsTool {
            disable_auto_readonly: Some(true),
            allow: vec!["ec2"].into_iter().collect(),
            ..Default::default()
        };
        let merged = child.merge(parent);
        assert_eq!(merged.aws.allow, vec!["ec2"].into_iter().collect());
        assert_eq!(merged.aws.deny, vec!["iam"].into_iter().collect());
        assert!(merged.aws.disable_auto_readonly.unwrap_or_default());
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_merge_shell() -> Result<()> {
        let child = ExecuteShellTool::default();
        let parent = ExecuteShellTool {
            deny_by_default: Some(false),
            disable_auto_readonly: Some(false),
            ..Default::default()
        };

        let merged = child.clone().merge(parent);
        assert!(!merged.deny_by_default.unwrap_or_default());
        assert!(!merged.disable_auto_readonly.unwrap_or_default());

        let parent = ExecuteShellTool {
            deny_by_default: Some(true),
            disable_auto_readonly: Some(true),
            ..Default::default()
        };
        let merged = child.clone().merge(parent);
        assert!(merged.deny_by_default.unwrap_or_default());
        assert!(merged.disable_auto_readonly.unwrap_or_default());

        let child = ExecuteShellTool {
            deny_by_default: Some(false),
            disable_auto_readonly: Some(false),
            ..Default::default()
        };
        let parent = ExecuteShellTool {
            deny_by_default: Some(true),
            disable_auto_readonly: Some(true),
            ..Default::default()
        };
        let merged = child.merge(parent);
        assert!(!merged.deny_by_default.unwrap_or_default());
        assert!(!merged.disable_auto_readonly.unwrap_or_default());
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_aws_kiro() -> Result<()> {
        let a = NativeTools::default();
        let kiro = KiroAwsTool::from(&a);
        assert!(kiro.auto_allow_readonly);
        assert!(kiro.allowed_services.is_empty());
        assert!(kiro.denied_services.is_empty());

        let a = NativeTools {
            aws: AwsTool {
                disable_auto_readonly: Some(true),
                allow: "blah".into(),
                deny: "blahblah".into(),
            },
            ..Default::default()
        };

        let kiro = KiroAwsTool::from(&a);
        assert!(!kiro.auto_allow_readonly);
        assert!(kiro.allowed_services.contains("blah"));
        assert!(kiro.denied_services.contains("blahblah"));
        assert_eq!(kiro.allowed_services.len() + kiro.denied_services.len(), 2);
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_shell_kiro() -> Result<()> {
        let a = NativeTools::default();
        let kiro = KiroShellTool::from(&a);
        assert!(kiro.auto_allow_readonly);
        assert!(kiro.allowed_commands.is_empty());
        assert!(kiro.denied_commands.is_empty());

        let a = NativeTools {
            shell: ExecuteShellTool {
                allow: "ls".into(),
                deny: "rm".into(),
                deny_by_default: None,
                disable_auto_readonly: None,
                override_command: HashSet::from_iter(vec!["rm".into()]),
            },
            ..Default::default()
        };
        let kiro = KiroShellTool::from(&a);
        assert!(kiro.auto_allow_readonly);
        assert_eq!(kiro.allowed_commands.len(), 2);
        assert_eq!(
            kiro.allowed_commands,
            HashSet::from_iter(vec!["ls".to_string(), "rm".to_string()])
        );
        assert!(kiro.denied_commands.is_empty());
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_read_kiro() -> Result<()> {
        let a = NativeTools::default();
        let kiro = KiroReadTool::from(&a);
        assert!(kiro.allowed_paths.is_empty());
        assert!(kiro.denied_paths.is_empty());

        let a = NativeTools {
            read: ReadTool {
                allow: "ls".into(),
                deny: "rm".into(),
                override_path: HashSet::from_iter(vec!["rm".into()]),
            },
            ..Default::default()
        };
        let kiro = KiroReadTool::from(&a);
        assert_eq!(kiro.allowed_paths.len(), 2);
        assert_eq!(
            kiro.allowed_paths,
            HashSet::from_iter(vec!["ls".to_string(), "rm".to_string()])
        );
        assert!(kiro.denied_paths.is_empty());
        Ok(())
    }

    #[test_log::test]
    pub fn test_native_write_kiro() -> Result<()> {
        let a = NativeTools::default();
        let kiro = KiroWriteTool::from(&a);
        assert!(kiro.allowed_paths.is_empty());
        assert!(kiro.denied_paths.is_empty());

        let a = NativeTools {
            write: WriteTool {
                allow: "ls".into(),
                deny: "rm".into(),
                override_path: HashSet::from_iter(vec!["rm".into()]),
            },
            ..Default::default()
        };
        let kiro = KiroWriteTool::from(&a);
        assert_eq!(kiro.allowed_paths.len(), 2);
        assert_eq!(
            kiro.allowed_paths,
            HashSet::from_iter(vec!["ls".to_string(), "rm".to_string()])
        );
        assert!(kiro.denied_paths.is_empty());
        Ok(())
    }
}
