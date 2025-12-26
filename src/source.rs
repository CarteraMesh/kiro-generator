use {
    super::*,
    crate::merging_format::MergingTomlFormat,
    colored::Colorize,
    config::FileSourceString,
    serde::Serialize,
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
        ops::{Deref, DerefMut},
        path::PathBuf,
    },
    super_table::Cell,
};

#[derive(Clone, Serialize)]
pub enum KdlAgentSource {
    LocalFile(PathBuf),
    LocalInline,
    GlobalFile(PathBuf),
    GlobalInline,
}

impl KdlAgentSource {
    fn is_local(&self) -> bool {
        matches!(self, Self::LocalFile(_) | Self::LocalInline)
    }
}

impl From<&KdlAgentSource> for Cell {
    fn from(value: &KdlAgentSource) -> Self {
        match value {
            KdlAgentSource::GlobalInline => Cell::new("global-inline"),
            KdlAgentSource::GlobalFile(p) => Cell::new(format!("{}", p.display())),
            KdlAgentSource::LocalInline => Cell::new("local-inline"),
            KdlAgentSource::LocalFile(p) => Cell::new(format!("{}", p.display())),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct KdlSources(pub HashMap<String, Vec<KdlAgentSource>>);
impl Debug for KdlSources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sources={}", self.0.len())
    }
}
impl From<&HashSet<String>> for KdlSources {
    fn from(value: &HashSet<String>) -> Self {
        let mut sources = Self(HashMap::with_capacity(value.len()));
        value.iter().for_each(|n| sources.add(n));
        sources
    }
}
impl KdlSources {
    pub fn is_local(&self, name: impl AsRef<str>) -> bool {
        if let Some(a) = self.get(name.as_ref()) {
            return a.iter().find(|p| p.is_local()).is_some();
        }
        false
    }

    fn add(&mut self, name: &str) {
        self.0.insert(name.to_string(), Vec::with_capacity(4));
    }
}

impl Deref for KdlSources {
    type Target = HashMap<String, Vec<KdlAgentSource>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KdlSources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub enum AgentSource {
    Raw(String),
    LocalFile(PathBuf),
    LocalInline(String),
    GlobalFile(PathBuf),
    GlobalInline(String),
}

impl Display for AgentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentSource::Raw(_) => write!(f, "raw"),
            AgentSource::GlobalInline(_) => write!(f, "global-inline",),
            AgentSource::LocalFile(path) => write!(f, "file://{}", path.display()),
            AgentSource::LocalInline(_) => write!(f, "local-inline",),
            AgentSource::GlobalFile(path) => write!(f, "file://{}", path.display()),
        }
    }
}

impl AgentSource {
    pub(super) fn to_cell(&self, color: bool, fs: &Fs) -> Cell {
        match self {
            AgentSource::Raw(_) => Cell::new(format!("{self}")),
            AgentSource::GlobalInline(content) | AgentSource::LocalInline(content) => {
                if content.is_empty() {
                    if color {
                        Cell::new(format!("{self} {}", "[empty]".red()))
                    } else {
                        Cell::new(format!("{self} empty"))
                    }
                } else {
                    Cell::new(format!("{self}"))
                }
            }
            AgentSource::GlobalFile(path) | AgentSource::LocalFile(path) => {
                match (fs.exists(path), fs.read_to_string_sync(path)) {
                    (true, Ok(_)) => Cell::new(format!("{self}")),
                    _ => {
                        if color {
                            Cell::new(format!("{self} {}", "[empty]".red()))
                        } else {
                            Cell::new(format!("{self} empty"))
                        }
                    }
                }
            }
        }
    }

    pub(super) fn new_local_agent(name: &str, fs: &Fs) -> (Self, bool) {
        let location = PathBuf::from(".kiro")
            .join("generators")
            .join(format!("{name}.toml"));
        let content = fs.read_to_string_sync(&location).ok().unwrap_or_default();
        (Self::LocalFile(location), content.is_empty())
    }

    pub(super) fn to_source(&self, fs: &Fs) -> config::File<FileSourceString, MergingTomlFormat> {
        match self {
            AgentSource::GlobalInline(content)
            | AgentSource::LocalInline(content)
            | AgentSource::Raw(content) => config::File::from_str(content, MergingTomlFormat),
            AgentSource::GlobalFile(path) | AgentSource::LocalFile(path) => {
                match (fs.exists(path), fs.read_to_string_sync(path)) {
                    (false, _) => config::File::from_str("", MergingTomlFormat),
                    (true, Ok(content)) => config::File::from_str(&content, MergingTomlFormat),
                    (true, Err(e)) => {
                        tracing::debug!("failed to read from file {}: {e}", path.display());
                        config::File::from_str("", MergingTomlFormat)
                    }
                }
            }
        }
    }
}
