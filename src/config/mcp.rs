use {crate::agent::CustomToolConfig, facet::Facet, facet_kdl as kdl};

#[derive(Facet, Clone, Debug)]
struct EnvVar {
    #[facet(kdl::argument)]
    key: String,
    #[facet(kdl::argument)]
    value: String,
}

#[derive(Facet, Clone, Debug)]
struct Header {
    #[facet(kdl::argument)]
    key: String,
    #[facet(kdl::argument)]
    value: String,
}

#[derive(Facet, Default, Clone, Debug)]
struct ToolArgs {
    #[facet(kdl::arguments, default)]
    args: Vec<String>,
}

#[derive(Facet, Clone, Debug, Eq, PartialEq)]
pub struct RedirectUri {
    #[facet(kdl::argument)]
    pub value: String,
}

#[derive(Facet, Clone, Debug)]
pub struct CustomToolConfigKdl {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::child, default)]
    pub url: Option<Url>,

    #[facet(kdl::child, default)]
    pub command: Option<Command>,

    #[facet(kdl::child, default)]
    args: Option<ToolArgs>,

    #[facet(kdl::children, default)]
    env: Vec<EnvVar>,

    #[facet(kdl::children, default)]
    header: Vec<Header>,

    #[facet(kdl::child, default)]
    pub timeout: Option<Timeout>,

    #[facet(kdl::child, default)]
    pub disabled: Option<Disabled>,
}

#[derive(Facet, Clone, Debug)]
pub struct Url {
    #[facet(kdl::argument)]
    pub value: String,
}

#[derive(Facet, Clone, Debug)]
pub struct Command {
    #[facet(kdl::argument)]
    pub value: String,
}

#[derive(Facet, Clone, Debug)]
pub struct Timeout {
    #[facet(kdl::argument)]
    pub value: u64,
}

#[derive(Facet, Clone, Debug)]
pub struct Disabled {
    #[facet(kdl::argument)]
    pub value: bool,
}

impl From<CustomToolConfigKdl> for CustomToolConfig {
    fn from(value: CustomToolConfigKdl) -> Self {
        let command = value.command.map(|c| c.value).unwrap_or_default();
        let url = value.url.map(|u| u.value).unwrap_or_default();

        Self {
            url,
            command,
            args: value.args.map(|a| a.args).unwrap_or_default(),
            timeout: value
                .timeout
                .map(|t| t.value)
                .filter(|&t| t != 0)
                .unwrap_or_else(crate::agent::tool_default_timeout),
            disabled: value.disabled.map(|d| d.value).unwrap_or(false),
            headers: value.header.into_iter().map(|h| (h.key, h.value)).collect(),
            env: value.env.into_iter().map(|e| (e.key, e.value)).collect(),
        }
    }
}

impl From<&CustomToolConfigKdl> for CustomToolConfig {
    fn from(value: &CustomToolConfigKdl) -> Self {
        value.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Facet, Debug)]
    struct McpDoc {
        #[facet(kdl::child)]
        mcp: CustomToolConfigKdl,
    }

    #[test]
    fn parse_basic_mcp() {
        let kdl = r#"mcp "rustdocs" {
            command "rust-docs-mcp"
            timeout 1000
        }"#;

        let doc: McpDoc = facet_kdl::from_str(kdl).unwrap();
        assert_eq!(doc.mcp.name, "rustdocs");
        assert_eq!(doc.mcp.command.unwrap().value, "rust-docs-mcp");
        assert_eq!(doc.mcp.timeout.unwrap().value, 1000);
    }

    #[test]
    fn parse_mcp_with_url() {
        let kdl = r#"mcp "remote" {
            url "http://localhost:8080"
        }"#;

        let doc: McpDoc = facet_kdl::from_str(kdl).unwrap();
        assert_eq!(doc.mcp.name, "remote");
        assert_eq!(doc.mcp.url.unwrap().value, "http://localhost:8080");
    }

    #[test]
    fn parse_mcp_with_env_and_headers() {
        let kdl = r#"mcp "api" {
            command "api-server"
            env "API_KEY" "secret123"
            env "DEBUG" "true"
            header "Authorization" "Bearer token"
            header "Content-Type" "application/json"
        }"#;

        let doc: McpDoc = facet_kdl::from_str(kdl).unwrap();
        assert_eq!(doc.mcp.env.len(), 2);
        assert_eq!(doc.mcp.header.len(), 2);
    }

    #[test]
    fn parse_mcp_with_args() {
        let kdl = r#"mcp "tool" {
            command "my-tool"
            args "--verbose" "--output" "json"
        }"#;

        let doc: McpDoc = facet_kdl::from_str(kdl).unwrap();
        assert_eq!(doc.mcp.args.unwrap().args, vec![
            "--verbose",
            "--output",
            "json"
        ]);
    }

    #[test]
    fn convert_to_custom_tool_config() {
        let kdl = r#"mcp "test" {
            command "test-cmd"
            timeout 5000
            disabled #true
        }"#;

        let doc: McpDoc = facet_kdl::from_str(kdl).unwrap();
        let config: CustomToolConfig = doc.mcp.into();

        assert_eq!(config.command, "test-cmd");
        assert_eq!(config.timeout, 5000);
        assert!(config.disabled);
    }

    #[test]
    fn default_timeout_when_zero() {
        let kdl = r#"mcp "test" {
            command "test-cmd"
            timeout 0
        }"#;

        let doc: McpDoc = facet_kdl::from_str(kdl).unwrap();
        let config: CustomToolConfig = doc.mcp.into();

        assert_eq!(config.timeout, crate::agent::tool_default_timeout());
    }
}
