mod agent;
mod agent_file;
mod hook;
mod mcp;
mod merge;
mod native;

use std::{collections::HashSet, fmt::Debug};

pub use {agent::KdlAgent, hook::HookPart, mcp::CustomToolConfigKdl, native::NativeTools};

#[derive(facet::Facet, Default)]
pub struct GeneratorConfig {
    #[facet(facet_kdl::children, default)]
    pub agent: Vec<KdlAgent>,
}

impl Debug for GeneratorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "agents={}", self.agent.len())
    }
}

impl GeneratorConfig {
    pub fn names(&self) -> HashSet<String> {
        self.agent.iter().map(|a| a.name.clone()).collect()
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&KdlAgent> {
        self.agent.iter().find(|a| a.name.eq(name.as_ref()))
    }
}
