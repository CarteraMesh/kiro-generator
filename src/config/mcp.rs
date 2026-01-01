use {
    super::IntDoc,
    crate::{agent::CustomToolConfig, config::GenericItem},
    facet::Facet,
    facet_kdl as kdl,
};

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

#[derive(Facet, Clone, Debug, Eq, PartialEq)]
pub struct RedirectUri {
    #[facet(kdl::argument)]
    pub value: String,
}

#[derive(Facet, Clone, Debug)]
pub struct Disabled {
    #[facet(kdl::argument)]
    pub value: bool,
}

#[derive(Facet, Default, Clone, Debug)]
#[facet(rename_all = "kebab-case", default)]
pub struct CustomToolConfigDoc {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::child, default)]
    pub url: Option<Url>,

    #[facet(kdl::child, default)]
    pub command: Option<Command>,

    #[facet(kdl::children, default)]
    args: Vec<GenericItem>,

    #[facet(kdl::children, default)]
    env: Vec<EnvVar>,

    #[facet(kdl::children, default)]
    header: Vec<Header>,

    #[facet(kdl::child, default)]
    pub(super) timeout: IntDoc,

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

impl From<CustomToolConfigDoc> for CustomToolConfig {
    fn from(value: CustomToolConfigDoc) -> Self {
        let command = value.command.map(|c| c.value).unwrap_or_default();
        let url = value.url.map(|u| u.value).unwrap_or_default();

        Self {
            url,
            command,
            args: value.args.into_iter().map(|v| v.item).collect(),
            timeout: if value.timeout.value == 0 {
                crate::agent::tool_default_timeout()
            } else {
                value.timeout.value
            },
            disabled: value.disabled.map(|d| d.value).unwrap_or(false),
            headers: value.header.into_iter().map(|h| (h.key, h.value)).collect(),
            env: value.env.into_iter().map(|e| (e.key, e.value)).collect(),
        }
    }
}

impl From<&CustomToolConfigDoc> for CustomToolConfig {
    fn from(value: &CustomToolConfigDoc) -> Self {
        value.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Facet, Debug)]
    struct McpDoc {
        #[facet(kdl::child)]
        mcp: CustomToolConfigDoc,
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
        assert_eq!(doc.mcp.timeout.value, 1000);
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
        let args: Vec<String> = doc.mcp.args.into_iter().map(|v| v.item).collect();
        assert_eq!(args, vec!["--verbose", "--output", "json"]);
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
