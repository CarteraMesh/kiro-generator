use {
    crate::agent::{
        AwsTool as KiroAwsTool,
        ExecuteShellTool as KiroShellTool,
        ReadTool as KiroReadTool,
        WriteTool as KiroWriteTool,
    },
    facet::Facet,
    facet_kdl as kdl,
    std::collections::HashSet,
};

#[derive(Facet, Debug, PartialEq, Clone, Eq, Hash)]
pub struct GenericListItem {
    #[facet(kdl::argument)]
    item: String,
}
macro_rules! define_tool {
    ($name:ident) => {
        #[derive(Clone, Debug, Default, PartialEq, Eq)]
        pub struct $name {
            pub allows: HashSet<String>,
            pub denies: HashSet<String>,
            pub overrides: HashSet<String>,
            pub disable_auto_readonly: Option<bool>,
            pub deny_by_default: Option<bool>,
        }

        impl $name {
            pub fn merge(mut self, other: Self) -> Self {
                self.allows.extend(other.allows);
                self.denies.extend(other.denies);
                self.disable_auto_readonly =
                    self.disable_auto_readonly.or(other.disable_auto_readonly);
                self.deny_by_default = self.deny_by_default.or(other.deny_by_default);
                self
            }
        }
    };
}

macro_rules! define_kdl_doc {
    ($name:ident) => {
        #[derive(Facet, Clone, Debug, Default, PartialEq, Eq)]
        #[facet(default, rename_all = "kebab-case")]
        pub struct $name {
            #[facet(default, kdl::children)]
            pub allows: Vec<GenericListItem>,
            #[facet(default, kdl::children)]
            pub denies: Vec<GenericListItem>,
            #[facet(default, kdl::children)]
            pub overrides: Vec<GenericListItem>,
            #[facet(default, kdl::property)]
            pub deny_by_default: Option<bool>,
            #[facet(default, kdl::property)]
            pub disable_auto_readonly: Option<bool>,
        }
    };
}

macro_rules! define_tool_into {
    ($name:ident, $to:ident) => {
        impl From<$name> for $to {
            fn from(value: $name) -> $to {
                $to {
                    allows: split_newline(value.allows),
                    denies: split_newline(value.denies),
                    overrides: split_newline(value.overrides),
                    deny_by_default: value.deny_by_default,
                    disable_auto_readonly: value.disable_auto_readonly,
                }
            }
        }
    };
}

define_kdl_doc!(AwsToolDoc);
define_kdl_doc!(ExecuteShellToolDoc);
define_kdl_doc!(WriteToolDoc);
define_kdl_doc!(ReadToolDoc);
define_tool!(ExecuteShellTool);
define_tool!(AwsTool);
define_tool!(WriteTool);
define_tool!(ReadTool);
define_tool_into!(ExecuteShellToolDoc, ExecuteShellTool);
define_tool_into!(AwsToolDoc, AwsTool);
define_tool_into!(WriteToolDoc, WriteTool);
define_tool_into!(ReadToolDoc, ReadTool);

fn split_newline(list: Vec<GenericListItem>) -> HashSet<String> {
    let values: Vec<&str> = list.iter().flat_map(|f| f.item.split("\n")).collect();
    let mut combined: Vec<String> = vec![];
    for v in values {
        combined.push(v.to_string());
    }
    HashSet::from_iter(combined)
}

#[derive(Facet, Default, Clone, Debug, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct NativeToolsDoc {
    #[facet(default, kdl::child)]
    pub shell: ExecuteShellToolDoc,
    #[facet(default, kdl::child)]
    pub aws: AwsToolDoc,
    #[facet(default, kdl::child)]
    pub read: ReadToolDoc,
    #[facet(default, kdl::child)]
    pub write: WriteToolDoc,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct NativeTools {
    pub shell: ExecuteShellTool,
    pub aws: AwsTool,
    pub read: ReadTool,
    pub write: WriteTool,
}

impl From<NativeToolsDoc> for NativeTools {
    fn from(value: NativeToolsDoc) -> Self {
        Self {
            shell: value.shell.into(),
            aws: value.aws.into(),
            read: value.read.into(),
            write: value.write.into(),
        }
    }
}

#[derive(Facet, Debug, Clone, Default, PartialEq, Eq)]
pub struct GenericList {
    #[facet(kdl::arguments)]
    pub list: Vec<String>,
}

impl From<&'static str> for GenericList {
    fn from(value: &'static str) -> Self {
        Self {
            list: vec![value.to_string()],
        }
    }
}

impl FromIterator<&'static str> for GenericList {
    fn from_iter<T: IntoIterator<Item = &'static str>>(iter: T) -> Self {
        Self {
            list: iter.into_iter().map(|f| f.to_string()).collect(),
        }
    }
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
        let aws = &value.aws;
        KiroAwsTool {
            allowed_services: aws.allows.clone(),
            denied_services: aws.denies.clone(),
            auto_allow_readonly: aws.disable_auto_readonly.unwrap_or(true),
        }
    }
}

impl From<&NativeTools> for KiroWriteTool {
    fn from(value: &NativeTools) -> Self {
        let write = &value.write;
        let mut allow: HashSet<String> = write.allows.clone();
        let mut deny: HashSet<String> = write.denies.clone();
        if !write.overrides.is_empty() {
            tracing::trace!(
                "Override/Forcing write: {:?}",
                write.overrides.iter().collect::<Vec<_>>()
            );
            for cmd in write.overrides.iter() {
                allow.insert(cmd.clone());
                if deny.remove(cmd) {
                    tracing::trace!("Removed from deny: {cmd}");
                }
            }
        }

        Self {
            allowed_paths: allow,
            denied_paths: deny,
        }
    }
}

impl From<&NativeTools> for KiroReadTool {
    fn from(value: &NativeTools) -> Self {
        let read = &value.read;
        let mut allow: HashSet<String> = read.allows.clone();
        let mut deny: HashSet<String> = read.denies.clone();
        if !read.overrides.is_empty() {
            tracing::trace!(
                "Override/Forcing write: {:?}",
                read.overrides.iter().collect::<Vec<_>>()
            );
            for cmd in read.overrides.iter() {
                allow.insert(cmd.clone());
                if deny.remove(cmd) {
                    tracing::trace!("Removed from deny: {cmd}");
                }
            }
        }

        Self {
            allowed_paths: allow,
            denied_paths: deny,
        }
    }
}

impl From<&NativeTools> for KiroShellTool {
    fn from(value: &NativeTools) -> Self {
        let shell = &value.shell;
        let mut allow: HashSet<String> = shell.allows.clone();
        let mut deny: HashSet<String> = shell.denies.clone();

        if !shell.overrides.is_empty() {
            tracing::trace!(
                "Override/Forcing commands: {:?}",
                shell.overrides.iter().collect::<Vec<_>>()
            );
            for cmd in shell.overrides.iter() {
                allow.insert(cmd.clone());
                if deny.remove(cmd) {
                    tracing::trace!("Removed command from deny: {cmd}");
                }
            }
        }
        Self {
            allowed_commands: allow,
            denied_commands: deny,
            deny_by_default: shell.deny_by_default.unwrap_or(false),
            auto_allow_readonly: shell.disable_auto_readonly.unwrap_or(true),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use {super::*, crate::Result};

//     #[derive(Facet, Debug)]
//     struct NativeToolsDoc {
//         #[facet(kdl::child)]
//         native: NativeTools,
//     }

//     #[test_log::test]
//     fn parse_shell_tool() {
//         let kdl = r#"native {
//             shell deny_by_default=#true disable_auto_readonly=#false {
//                 allow "ls .*" "git status"
//                 deny "rm -rf /"
//                 override "git push"
//             }
//         }"#;

//         let doc: NativeToolsDoc = facet_kdl::from_str(kdl).unwrap();
//         let shell = doc.native.shell.unwrap();
//         assert_eq!(shell.allow.unwrap().list.len(), 2);
//         assert_eq!(shell.deny.unwrap().list.len(), 1);
//         assert!(shell.deny_by_default.unwrap());
//         assert!(!shell.disable_auto_readonly.unwrap());
//         assert_eq!(shell.r#override.len(), 1);
//     }

//     #[test_log::test]
//     fn parse_aws_tool() {
//         let kdl = r#"native {
//             aws disable_auto_readonly=#true {
//                 allow "ec2" "s3"
//                 deny "iam"
//             }
//         }"#;

//         let doc: NativeToolsDoc = facet_kdl::from_str(kdl).unwrap();
//         let aws = doc.native.aws.unwrap();
//         assert!(aws.disable_auto_readonly.unwrap());
//         assert_eq!(aws.allow.unwrap().list.len(), 2);
//         assert_eq!(aws.deny.unwrap().list.len(), 1);
//     }

//     #[test_log::test]
//     fn parse_read_write_tools() {
//         let kdl = r#"native {
//             read {
//                 allow "*.rs" "*.toml"
//                 deny "/etc/*"
//                 override "/etc/hosts"
//             }
//             write {
//                 allow "*.txt"
//                 deny "/tmp/*"
//                 override "/tmp/allowed"
//             }
//         }"#;

//         let doc: NativeToolsDoc = facet_kdl::from_str(kdl).unwrap();
//         assert_eq!(doc.native.read.unwrap().allow.unwrap().list.len(), 2);
//         assert_eq!(doc.native.write.unwrap().allow.unwrap().list.len(), 1);
//     }

//     #[test_log::test]
//     pub fn test_native_merge_empty() -> Result<()> {
//         let child = NativeTools::default();
//         let parent = NativeTools::default();
//         let merged = child.merge(parent);

//         assert_eq!(merged, NativeTools::default());
//         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_native_merge_empty_child() -> Result<()> {
//         let child = NativeTools::default();
//         let parent = NativeTools {
//             aws: Some(AwsTool {
//                 disable_auto_readonly: None,
//                 allow: Some(vec!["ec2"].into_iter().collect()),
//                 deny: Some(vec!["iam"].into_iter().collect()),
//             }),
//             shell: Some(ExecuteShellTool {
//                 allow: Some(vec!["ls .*"].into_iter().collect()),
//                 deny: Some(vec!["git push"].into_iter().collect()),
//                 r#override: vec![Override::from("rm -rf /")],
//                 deny_by_default: Some(true),
//                 disable_auto_readonly: Some(false),
//             }),
//             read: Some(ReadTool {
//                 allow: Some(vec!["ls .*"].into_iter().collect()),
//                 deny: Some(vec!["git push"].into_iter().collect()),
//                 r#override: vec![Override::from("rm -rf /")],
//             }),
//             write: Some(WriteTool {
//                 allow: Some(vec!["ls .*"].into_iter().collect()),
//                 deny: Some(vec!["git push"].into_iter().collect()),
//                 overrides: vec![Override::from("rm -rf /")],
//             }),
//         };

//         let merged = child.merge(parent.clone());
//         assert_eq!(merged.aws, parent.aws);
//         assert_eq!(merged.shell, parent.shell);
//         assert_eq!(merged.read, parent.read);
//         assert_eq!(merged.write, parent.write);
//         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_native_merge_child_parent() -> Result<()> {
//         let child = NativeTools {
//             aws: Some(AwsTool {
//                 disable_auto_readonly: Some(true),
//                 allow: Some(vec!["ec2"].into_iter().collect()),
//                 deny: None,
//             }),
//             ..Default::default()
//         };

//         let parent = NativeTools {
//             aws: Some(AwsTool {
//                 disable_auto_readonly: None,
//                 allow: Some(vec!["ec2"].into_iter().collect()),
//                 deny: Some(vec!["iam"].into_iter().collect()),
//             }),
//             ..Default::default()
//         };

//         let merged = child.merge(parent);
//         let aws = merged.aws.unwrap();
//         assert!(aws.disable_auto_readonly.unwrap());
//         // Should have deduplicated ec2
//         assert_eq!(aws.allow.unwrap().into_set().len(), 1);
//         assert_eq!(
//             aws.deny.unwrap().into_set(),
//             HashSet::from_iter(vec!["iam".to_string()])
//         );
//         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_native_merge_shell() -> Result<()> {
//         let child = ExecuteShellTool::default();
//         let parent = ExecuteShellTool {
//             deny_by_default: Some(false),
//             disable_auto_readonly: Some(false),
//             ..Default::default()
//         };

//         let merged = child.clone().merge(parent);
//         assert!(!merged.deny_by_default.unwrap());
//         assert!(!merged.disable_auto_readonly.unwrap());

//         let parent = ExecuteShellTool {
//             deny_by_default: Some(true),
//             disable_auto_readonly: Some(true),
//             ..Default::default()
//         };
//         let merged = child.clone().merge(parent);
//         assert!(merged.deny_by_default.unwrap());
//         assert!(merged.disable_auto_readonly.unwrap());

//         let child = ExecuteShellTool {
//             deny_by_default: Some(false),
//             disable_auto_readonly: Some(false),
//             ..Default::default()
//         };
//         let parent = ExecuteShellTool {
//             deny_by_default: Some(true),
//             disable_auto_readonly: Some(true),
//             ..Default::default()
//         };
//         let merged = child.merge(parent);
//         assert!(!merged.deny_by_default.unwrap());
//         assert!(!merged.disable_auto_readonly.unwrap());
//         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_native_aws_kiro() -> Result<()> {
//         let a = NativeTools::default();
//         let kiro = KiroAwsTool::from(&a);
//         assert!(kiro.auto_allow_readonly);
//         assert!(kiro.allowed_services.is_empty());
//         assert!(kiro.denied_services.is_empty());

//         let a = NativeTools {
//             aws: Some(AwsTool {
//                 disable_auto_readonly: Some(true),
//                 allow: Some("blah".into()),
//                 deny: Some("blahblah".into()),
//             }),
//             ..Default::default()
//         };

//         let kiro = KiroAwsTool::from(&a);
//         assert!(!kiro.auto_allow_readonly);
//         assert!(kiro.allowed_services.contains("blah"));
//         assert!(kiro.denied_services.contains("blahblah"));
//         assert_eq!(kiro.allowed_services.len() + kiro.denied_services.len(),
// 2);         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_native_shell_kiro() -> Result<()> {
//         let a = NativeTools::default();
//         let kiro = KiroShellTool::from(&a);
//         assert!(kiro.auto_allow_readonly);
//         assert!(kiro.allowed_commands.is_empty());
//         assert!(kiro.denied_commands.is_empty());

//         let a = NativeTools {
//             shell: Some(ExecuteShellTool {
//                 allow: Some("ls".into()),
//                 deny: Some("rm".into()),
//                 deny_by_default: None,
//                 disable_auto_readonly: None,
//                 r#override: vec!["rm".into()],
//             }),
//             ..Default::default()
//         };
//         let kiro = KiroShellTool::from(&a);
//         assert!(kiro.auto_allow_readonly);
//         assert_eq!(kiro.allowed_commands.len(), 2);
//         assert_eq!(
//             kiro.allowed_commands,
//             HashSet::from_iter(vec!["ls".to_string(), "rm".to_string()])
//         );
//         assert!(kiro.denied_commands.is_empty());
//         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_native_read_kiro() -> Result<()> {
//         let a = NativeTools::default();
//         let kiro = KiroReadTool::from(&a);
//         assert!(kiro.allowed_paths.is_empty());
//         assert!(kiro.denied_paths.is_empty());

//         let a = NativeTools {
//             read: Some(ReadTool {
//                 allow: Some("ls".into()),
//                 deny: Some("rm".into()),
//                 r#override: vec!["rm".into()],
//             }),
//             ..Default::default()
//         };
//         let kiro = KiroReadTool::from(&a);
//         assert_eq!(kiro.allowed_paths.len(), 2);
//         assert_eq!(
//             kiro.allowed_paths,
//             HashSet::from_iter(vec!["ls".to_string(), "rm".to_string()])
//         );
//         assert!(kiro.denied_paths.is_empty());
//         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_native_write_kiro() -> Result<()> {
//         let a = NativeTools::default();
//         let kiro = KiroWriteTool::from(&a);
//         assert!(kiro.allowed_paths.is_empty());
//         assert!(kiro.denied_paths.is_empty());

//         let a = NativeTools {
//             write: Some(WriteTool {
//                 allow: Some("ls".into()),
//                 deny: Some("rm".into()),
//                 overrides: vec!["rm".into()],
//             }),
//             ..Default::default()
//         };
//         let kiro = KiroWriteTool::from(&a);
//         assert_eq!(kiro.allowed_paths.len(), 2);
//         assert_eq!(
//             kiro.allowed_paths,
//             HashSet::from_iter(vec!["ls".to_string(), "rm".to_string()])
//         );
//         assert!(kiro.denied_paths.is_empty());
//         Ok(())
//     }
// }
