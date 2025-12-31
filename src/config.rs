mod agent;
mod agent_file;
mod hook;
mod mcp;
mod merge;
mod native;

use std::{collections::HashMap, fmt::Debug};

pub use agent::{KdlAgent, KdlAgentDoc};

#[derive(facet::Facet, Default)]
pub struct GeneratorConfigDoc {
    #[facet(facet_kdl::children, default)]
    pub agent: Vec<KdlAgentDoc>,
}

#[derive(Default)]
pub struct GeneratorConfig {
    pub agent: HashMap<String, KdlAgent>,
}

impl From<GeneratorConfigDoc> for GeneratorConfig {
    fn from(value: GeneratorConfigDoc) -> Self {
        let mut agent: HashMap<String, KdlAgent> = HashMap::with_capacity(value.agent.len());
        for a in value.agent {
            agent.insert(a.name.clone(), a.into());
        }
        Self { agent }
    }
}

impl Debug for GeneratorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "agents={}", self.agent.len())
    }
}

impl GeneratorConfig {
    pub fn get(&self, name: impl AsRef<str>) -> Option<&KdlAgent> {
        self.agent.get(name.as_ref())
    }
}
