use {crate::agent::CustomToolConfig, knuffel::Decode};

#[derive(Decode, Clone, Debug)]
struct EnvVar {
    #[knuffel(argument)]
    key: String,
    #[knuffel(argument)]
    value: String,
}

#[derive(Decode, Clone, Debug)]
struct Header {
    #[knuffel(argument)]
    key: String,
    #[knuffel(argument)]
    value: String,
}

#[derive(Decode, Default, Clone, Debug)]
struct ToolArgs {
    #[knuffel(arguments, default)]
    args: Vec<String>,
}

#[derive(Decode, Clone, Debug, Eq, PartialEq)]
pub struct OAuthConfig {
    /// Custom redirect URI for OAuth flow (e.g., "127.0.0.1:7778")
    /// If not specified, a random available port will be assigned by the OS
    #[knuffel(child, unwrap(argument))]
    pub redirect_uri: String,
}

#[derive(Decode, Clone, Debug)]
pub struct CustomToolConfigKdl {
    #[knuffel(argument)]
    pub name: String,

    #[knuffel(child, unwrap(argument))]
    pub url: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub command: Option<String>,

    #[knuffel(child)]
    pub oauth: Option<OAuthConfig>,

    #[knuffel(child, default)]
    args: ToolArgs,

    #[knuffel(children(name = "env"))]
    env_vars: Vec<EnvVar>,

    #[knuffel(children(name = "header"))]
    headers: Vec<Header>,

    #[knuffel(child, default, unwrap(argument))]
    pub timeout: u64,

    #[knuffel(child, default, unwrap(argument))]
    pub disabled: bool,
}

impl From<CustomToolConfigKdl> for CustomToolConfig {
    fn from(value: CustomToolConfigKdl) -> Self {
        let command = value.command.unwrap_or_default();
        let oauth = value.oauth.map(|o| crate::agent::OAuthConfig {
            redirect_uri: Some(o.redirect_uri),
        });
        let url = value.url.unwrap_or_default();

        Self {
            url,
            r#type: if command.is_empty() {
                crate::agent::TransportType::Stdio
            } else {
                crate::agent::TransportType::Http
            },
            command,
            args: value.args.args,
            oauth,
            timeout: if value.timeout == 0 {
                crate::agent::tool_default_timeout()
            } else {
                value.timeout
            },
            disabled: value.disabled,
            headers: value
                .headers
                .into_iter()
                .map(|h| (h.key, h.value))
                .collect(),
            env: value
                .env_vars
                .into_iter()
                .map(|e| (e.key, e.value))
                .collect(),
        }
    }
}
