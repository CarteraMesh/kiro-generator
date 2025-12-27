use {
    serde::{Deserialize, Serialize},
    std::{collections::HashSet, fmt::Display},
};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, enum_iterator::Sequence,
)]
#[serde(rename_all = "camelCase")]
pub enum ToolTarget {
    Aws,
    Shell,
    Read,
    Write,
}

impl Display for ToolTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolTarget::Aws => write!(f, "aws"),
            ToolTarget::Shell => write!(f, "shell"),
            ToolTarget::Read => write!(f, "read"),
            ToolTarget::Write => write!(f, "write"),
        }
    }
}

impl AsRef<str> for ToolTarget {
    fn as_ref(&self) -> &str {
        match self {
            ToolTarget::Aws => "aws",
            ToolTarget::Shell => "shell",
            ToolTarget::Read => "read",
            ToolTarget::Write => "write",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AwsTool {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub allowed_services: HashSet<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub denied_services: HashSet<String>,
    #[serde(default)]
    pub auto_allow_readonly: bool,
}

impl Default for AwsTool {
    fn default() -> Self {
        Self {
            allowed_services: Default::default(),
            denied_services: Default::default(),
            auto_allow_readonly: true,
        }
    }
}

fn default_allow_read_only() -> bool {
    false
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecuteShellTool {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub allowed_commands: HashSet<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub denied_commands: HashSet<String>,
    #[serde(default)]
    pub deny_by_default: bool,
    #[serde(default = "default_allow_read_only")]
    pub auto_allow_readonly: bool,
}

impl Default for ExecuteShellTool {
    fn default() -> Self {
        Self {
            allowed_commands: Default::default(),
            denied_commands: Default::default(),
            deny_by_default: false,
            auto_allow_readonly: default_allow_read_only(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadTool {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub allowed_paths: HashSet<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub denied_paths: HashSet<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WriteTool {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub allowed_paths: HashSet<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub denied_paths: HashSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_target_display() {
        assert_eq!(ToolTarget::Aws.to_string(), "aws");
        assert_eq!(ToolTarget::Shell.to_string(), "shell");
    }

    #[test]
    fn tool_target_as_ref() {
        assert_eq!(ToolTarget::Read.as_ref(), "read");
        assert_eq!(ToolTarget::Write.as_ref(), "write");
    }

    #[test]
    fn aws_tool_default() {
        let tool = AwsTool::default();
        assert!(tool.auto_allow_readonly);
        assert!(tool.allowed_services.is_empty());
    }

    #[test]
    fn execute_shell_tool_default() {
        let tool = ExecuteShellTool::default();
        assert!(!tool.deny_by_default);
        assert!(!tool.auto_allow_readonly);
    }

    #[test]
    fn read_tool_serde() {
        let tool = ReadTool::default();
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: ReadTool = serde_json::from_str(&json).unwrap();
        assert_eq!(tool, deserialized);
    }

    #[test]
    fn write_tool_serde() {
        let tool = WriteTool::default();
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: WriteTool = serde_json::from_str(&json).unwrap();
        assert_eq!(tool, deserialized);
    }
}
