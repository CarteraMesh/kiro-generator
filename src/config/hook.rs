use {
    crate::agent::hook::{Hook, HookTrigger},
    facet::Facet,
    facet_kdl as kdl,
    std::collections::HashMap,
};

#[derive(Facet, Clone, Debug, PartialEq, Eq)]
struct Command {
    #[facet(kdl::argument)]
    value: String,
}

#[derive(Facet, Clone, Debug, PartialEq, Eq)]
struct TimeoutMs {
    #[facet(kdl::argument)]
    value: usize,
}

#[derive(Facet, Clone, Debug, PartialEq, Eq)]
struct MaxOutputSize {
    #[facet(kdl::argument)]
    value: usize,
}

#[derive(Facet, Clone, Debug, PartialEq, Eq)]
struct CacheTtlSeconds {
    #[facet(kdl::argument)]
    value: usize,
}

#[derive(Facet, Clone, Debug, PartialEq, Eq)]
struct Matcher {
    #[facet(kdl::argument)]
    value: String,
}

macro_rules! define_hook_doc {
    ($name:ident) => {
        #[derive(Facet, Default, Clone, Debug, PartialEq, Eq)]
        #[facet(default, rename_all = "kebab-case")]
        pub struct $name {
            #[facet(kdl::argument)]
            pub name: String,
            #[facet(kdl::child, default)]
            command: GenericValue,
            #[facet(default, kdl::child)]
            timeout_ms: IntDoc,
            #[facet(kdl::child, default)]
            max_output_size: IntDoc,
            #[facet(kdl::child, default)]
            cache_ttl_seconds: IntDoc,
            #[facet(kdl::child, default)]
            matcher: Option<GenericValue>,
        }
        impl From<$name> for Hook {
            fn from(value: $name) -> Hook {
                Hook {
                    command: value.command.value,
                    timeout_ms: value.timeout_ms.value as u64,
                    max_output_size: value.max_output_size.value,
                    cache_ttl_seconds: value.cache_ttl_seconds.value as u64,
                    matcher: value.matcher.map(|m| m.value),
                }
            }
        }
    };
}

macro_rules! define_hook {
    ($name:ident) => {
        #[derive(Default, Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            #[facet(kdl::argument)]
            pub name: String,
            command: String,
            timeout_ms: u64,
            max_output_size: u64,
            cache_ttl_seconds: u64,
            matcher: Option<String>,
        }
    };
}

#[derive(Facet, Clone, Default, Debug, PartialEq, Eq)]
#[facet(default)]
struct GenericValue {
    #[facet(kdl::argument)]
    value: String,
}

#[derive(Facet, Default, Clone, Debug, PartialEq, Eq)]
#[facet(default)]
struct IntDoc {
    #[facet(kdl::argument)]
    value: usize,
}

define_hook_doc!(HookAgentSpawnDoc);
define_hook_doc!(HookUserPromptSubmitDoc);
define_hook_doc!(HookPreToolUseDoc);
define_hook_doc!(HookPostToolUseDoc);
define_hook_doc!(HookStopDoc);

#[derive(Facet, Clone, Default, Debug, PartialEq, Eq)]
#[facet(default, rename_all = "kebab-case")]
pub struct HookDoc {
    #[facet(kdl::children, default)]
    pub agent_spawn: Vec<HookAgentSpawnDoc>,
    #[facet(kdl::children, default)]
    pub user_prompt_submit: Vec<HookUserPromptSubmitDoc>,
    #[facet(kdl::children, default)]
    pub pre_tool_use: Vec<HookPreToolUseDoc>,
    #[facet(kdl::children, default)]
    pub post_tool_use: Vec<HookPostToolUseDoc>,
    #[facet(kdl::children, default)]
    pub stop: Vec<HookStopDoc>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct HookPart {
    pub agent_spawn: HashMap<String, Hook>,
    pub user_prompt_submit: HashMap<String, Hook>,
    pub pre_tool_use: HashMap<String, Hook>,
    pub post_tool_use: HashMap<String, Hook>,
    pub stop: HashMap<String, Hook>,
}

impl From<HookDoc> for HookPart {
    fn from(value: HookDoc) -> Self {
        Self {
            agent_spawn: value
                .agent_spawn
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
            user_prompt_submit: value
                .user_prompt_submit
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
            pre_tool_use: value
                .pre_tool_use
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
            post_tool_use: value
                .post_tool_use
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
            stop: value
                .stop
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
        }
    }
}
impl HookPart {
    pub fn merge(mut self, other: Self) -> Self {
        match (self.agent_spawn.is_empty(), other.agent_spawn.is_empty()) {
            (false, false) => {
                let mut hooks = HashMap::with_capacity(self.agent_spawn.len());
                for (k, h) in self.agent_spawn {
                    if let Some(o) = other.agent_spawn.get(&k) {
                        hooks.insert(k.to_string(), h.merge(o.clone()));
                    } else {
                        hooks.insert(k, h);
                    }
                }
                self.agent_spawn = hooks;
                for o in other.agent_spawn.keys() {
                    if !self.agent_spawn.contains_key(o) {
                        self.agent_spawn
                            .insert(o.to_string(), other.agent_spawn.get(o).unwrap().clone());
                    }
                }
            }
            (true, false) => self.agent_spawn = other.agent_spawn,
            _ => {}
        };
        self
    }
}

// #[cfg(test)]
// mod tests {
//     use {super::*, crate::Result, std::time::Duration};

//     macro_rules! rando_hook {
//         ($name:ident) => {
//             impl $name {
//                 fn rando() -> $name {
//                     let value = std::time::SystemTime::now()
//                         .duration_since(std::time::UNIX_EPOCH)
//                         .unwrap()
//                         .as_secs();
//                     Self {
//                         name: format!("$name-{value}"),
//                         command: Some(Command {
//                             value: format!("{value}"),
//                         }),
//                         timeout_ms: Some(TimeoutMs { value }),
//                         max_output_size: None,
//                         cache_ttl_seconds: Some(CacheTtlSeconds { value }),
//                         matcher: Some(Matcher {
//                             value: format!("{value}"),
//                         }),
//                     }
//                 }
//             }
//         };
//     }
//     rando_hook!(HookAgentSpawn);
//     rando_hook!(HookUserPromptSubmit);
//     rando_hook!(HookPreToolUse);
//     rando_hook!(HookPostToolUse);
//     rando_hook!(HookStop);

//     impl HookPart {
//         pub fn randomize() -> Self {
//             Self {
//                 agent_spawn: vec![HookAgentSpawn::rando()],
//                 user_prompt_submit: vec![HookUserPromptSubmit::rando()],
//                 pre_tool_use: vec![HookPreToolUse::rando()],
//                 post_tool_use: vec![HookPostToolUse::rando()],
//                 stop: vec![HookStop::rando()],
//             }
//         }
//     }

//     #[test_log::test]
//     pub fn test_hooks_empty() -> Result<()> {
//         let child = HookPart::default();
//         let parent = HookPart::default();
//         let merged = child.merge(parent);

//         assert!(merged.agent_spawn.is_empty());
//         assert!(merged.user_prompt_submit.is_empty());
//         assert!(merged.pre_tool_use.is_empty());
//         assert!(merged.post_tool_use.is_empty());
//         assert!(merged.stop.is_empty());
//         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_hooks_empty_child() -> Result<()> {
//         let child = HookPart::default();
//         let parent = HookPart::randomize();
//         let before = parent.clone();
//         let merged = child.merge(parent);

//         assert_eq!(merged, before);
//         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_hooks_no_merge() -> Result<()> {
//         let child = HookPart::randomize();
//         let parent = HookPart::randomize();
//         let before = child.clone();
//         let merged = child.merge(parent);
//         assert_eq!(merged, before);
//         Ok(())
//     }

//     #[test_log::test]
//     pub fn test_hooks_merge_parent() -> Result<()> {
//         let child = HookPart::randomize();
//         std::thread::sleep(Duration::from_millis(1300));
//         let parent = HookPart::randomize();
//         let merged = child.merge(parent);
//         assert_eq!(merged.agent_spawn.len(), 2);
//         assert_eq!(merged.user_prompt_submit.len(), 2);
//         assert_eq!(merged.pre_tool_use.len(), 2);
//         assert_eq!(merged.post_tool_use.len(), 2);
//         assert_eq!(merged.stop.len(), 2);
//         Ok(())
//     }
// }
