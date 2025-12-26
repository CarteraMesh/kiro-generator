use {
    super::custom_tool::CustomToolConfig,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "camelCase", transparent)]
pub struct McpServerConfig {
    pub mcp_servers: HashMap<String, CustomToolConfig>,
}
