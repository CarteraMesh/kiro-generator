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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_server_config_default() {
        let config = McpServerConfig::default();
        assert!(config.mcp_servers.is_empty());
    }

    #[test]
    fn mcp_server_config_serde() {
        let mut config = McpServerConfig::default();
        config.mcp_servers.insert("test".into(), CustomToolConfig {
            url: String::new(),
            headers: HashMap::new(),
            command: "cmd".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: 120_000,
            disabled: false,
        });
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: McpServerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
