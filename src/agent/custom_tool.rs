use {
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Clone, Default, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TransportType {
    /// Standard input/output transport (default)
    #[default]
    Stdio,
    /// HTTP transport for web-based communication
    Http,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OAuthConfig {
    /// Custom redirect URI for OAuth flow (e.g., "127.0.0.1:7778")
    /// If not specified, a random available port will be assigned by the OS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CustomToolConfig {
    /// The transport type to use for communication with the MCP server
    #[serde(default)]
    pub r#type: TransportType,
    /// The URL for HTTP-based MCP server communication
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// HTTP headers to include when communicating with HTTP-based MCP servers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// OAuth configuration for this server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<OAuthConfig>,
    /// The command string used to initialize the mcp server
    #[serde(default)]
    pub command: String,
    /// A list of arguments to be used to run the command with
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// A list of environment variables to run the command with
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// Timeout for each mcp request in ms
    #[serde(default = "tool_default_timeout")]
    pub timeout: u64,
    /// A boolean flag to denote whether or not to load this mcp server
    #[serde(default)]
    pub disabled: bool,
}

pub fn tool_default_timeout() -> u64 {
    120 * 1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_type_default() {
        assert_eq!(tool_default_timeout(), 120 * 1000);
        assert_eq!(TransportType::default(), TransportType::Stdio);
    }

    #[test]
    fn custom_tool_config_serde() {
        let config = CustomToolConfig {
            r#type: TransportType::Http,
            url: "http://test".into(),
            headers: HashMap::new(),
            oauth: None,
            command: "cmd".into(),
            args: vec!["arg1".into()],
            env: HashMap::new(),
            timeout: 5000,
            disabled: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CustomToolConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn oauth_config_serde() {
        let oauth = OAuthConfig {
            redirect_uri: Some("localhost:8080".into()),
        };
        let json = serde_json::to_string(&oauth).unwrap();
        let deserialized: OAuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(oauth, deserialized);
    }
}
