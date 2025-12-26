use {
    crate::agent::hook::{Hook, HookTrigger},
    knuffel::Decode,
    std::collections::HashMap,
};

macro_rules! define_hook {
    ($name:ident) => {
        #[derive(Decode, Default, Clone, Debug, PartialEq, Eq)]
        pub(super) struct $name {
            /// The command to run when the hook is triggered
            #[knuffel(argument)]
            pub name: String,
            /// The command to run when the hook is triggered
            #[knuffel(child, default, unwrap(argument))]
            pub command: String,

            /// Max time the hook can run before it throws a timeout error
            #[knuffel(child, default, unwrap(argument))]
            pub timeout_ms: u64,

            /// Max output size of the hook before it is truncated
            #[knuffel(child, default, unwrap(argument))]
            pub max_output_size: usize,

            /// How long the hook output is cached before it will be executed again
            #[knuffel(child, default, unwrap(argument))]
            pub cache_ttl_seconds: u64,

            /// Optional glob matcher for hook
            /// Currently used for matching tool name of PreToolUse and PostToolUse hook
            #[knuffel(child, default, unwrap(argument))]
            pub matcher: Option<String>,
        }

        impl From<$name> for Hook {
            fn from(value: $name) -> Hook {
                Hook {
                    command: value.command,
                    timeout_ms: value.timeout_ms,
                    max_output_size: value.max_output_size,
                    cache_ttl_seconds: value.cache_ttl_seconds,
                    matcher: value.matcher,
                }
            }
        }

        impl $name {
            fn merge(mut self, o: $name) -> $name {
                if self.cache_ttl_seconds == 0 && o.cache_ttl_seconds > 0 {
                    self.cache_ttl_seconds = o.cache_ttl_seconds;
                }
                if self.command.is_empty() {
                    self.command = o.command.clone();
                }
                if self.max_output_size == 0 && o.max_output_size > 0 {
                    self.max_output_size = o.max_output_size;
                }
                if self.timeout_ms == 0 && o.timeout_ms > 0 {
                    self.timeout_ms = o.timeout_ms;
                }
                if self.matcher.is_none()
                    && let Some(m) = &o.matcher
                {
                    self.matcher = Some(m.clone());
                }
                self
            }
        }
    };
}

define_hook!(HookAgentSpawn);
define_hook!(HookUserPromptSubmit);
define_hook!(HookPreToolUse);
define_hook!(HookPostToolUse);
define_hook!(HookStop);

#[derive(Decode, Clone, Default, Debug, PartialEq, Eq)]
pub(super) struct HookPart {
    #[knuffel(children(name = "agent-spawn"), default)]
    pub agent_spawn: Vec<HookAgentSpawn>,
    #[knuffel(children(name = "user-prompt-submit"), default)]
    pub user_prompt_submit: Vec<HookUserPromptSubmit>,
    #[knuffel(children(name = "pre-tool-use"), default)]
    pub pre_tool_use: Vec<HookPreToolUse>,
    #[knuffel(children(name = "post-tool-use"), default)]
    pub post_tool_use: Vec<HookPostToolUse>,
    #[knuffel(children(name = "stop"), default)]
    pub stop: Vec<HookStop>,
}

impl HookPart {
    pub fn merge(mut self, other: Self) -> Self {
        match (self.agent_spawn.is_empty(), other.agent_spawn.is_empty()) {
            (false, false) => {
                let mut hooks = Vec::with_capacity(self.agent_spawn.len());
                for h in self.agent_spawn {
                    if let Some(o) = other.agent_spawn.iter().find(|i| i.name == h.name) {
                        hooks.push(h.merge(o.clone()));
                    } else {
                        hooks.push(h);
                    }
                }
                self.agent_spawn = hooks;
                for o in other.agent_spawn.into_iter() {
                    if !self.agent_spawn.iter().any(|h| h.name == o.name) {
                        self.agent_spawn.push(o);
                    }
                }
            }
            (true, false) => self.agent_spawn = other.agent_spawn,
            _ => {}
        };

        match (
            self.user_prompt_submit.is_empty(),
            other.user_prompt_submit.is_empty(),
        ) {
            (false, false) => {
                let mut hooks = Vec::with_capacity(self.user_prompt_submit.len());
                for h in self.user_prompt_submit {
                    if let Some(o) = other.user_prompt_submit.iter().find(|i| i.name.eq(&h.name)) {
                        hooks.push(h.merge(o.clone()));
                    } else {
                        hooks.push(h);
                    }
                }
                self.user_prompt_submit = hooks;
                for o in other.user_prompt_submit.into_iter() {
                    if !self.user_prompt_submit.iter().any(|h| h.name == o.name) {
                        self.user_prompt_submit.push(o);
                    }
                }
            }
            (true, false) => self.user_prompt_submit = other.user_prompt_submit,
            _ => {}
        };

        match (self.pre_tool_use.is_empty(), other.pre_tool_use.is_empty()) {
            (false, false) => {
                let mut hooks = Vec::with_capacity(self.pre_tool_use.len());
                for h in self.pre_tool_use {
                    if let Some(o) = other.pre_tool_use.iter().find(|i| i.name.eq(&h.name)) {
                        hooks.push(h.merge(o.clone()));
                    } else {
                        hooks.push(h);
                    }
                }
                self.pre_tool_use = hooks;
                for o in other.pre_tool_use.into_iter() {
                    if !self.pre_tool_use.iter().any(|h| h.name == o.name) {
                        self.pre_tool_use.push(o);
                    }
                }
            }
            (true, false) => self.pre_tool_use = other.pre_tool_use,
            _ => {}
        };

        match (
            self.post_tool_use.is_empty(),
            other.post_tool_use.is_empty(),
        ) {
            (false, false) => {
                let mut hooks = Vec::with_capacity(self.post_tool_use.len());
                for h in self.post_tool_use {
                    if let Some(o) = other.post_tool_use.iter().find(|i| i.name.eq(&h.name)) {
                        hooks.push(h.merge(o.clone()));
                    } else {
                        hooks.push(h);
                    }
                }
                self.post_tool_use = hooks;
                for o in other.post_tool_use.into_iter() {
                    if !self.post_tool_use.iter().any(|h| h.name == o.name) {
                        self.post_tool_use.push(o);
                    }
                }
            }
            (true, false) => self.post_tool_use = other.post_tool_use,
            _ => {}
        };

        match (self.stop.is_empty(), other.stop.is_empty()) {
            (false, false) => {
                let mut hooks = Vec::with_capacity(self.stop.len());
                for h in self.stop {
                    if let Some(o) = other.stop.iter().find(|i| i.name.eq(&h.name)) {
                        hooks.push(h.merge(o.clone()));
                    } else {
                        hooks.push(h);
                    }
                }
                self.stop = hooks;
                for o in other.stop.into_iter() {
                    if !self.stop.iter().any(|h| h.name == o.name) {
                        self.stop.push(o);
                    }
                }
            }
            (true, false) => self.stop = other.stop,
            _ => {}
        };
        self
    }

    pub fn triggers(&self) -> HashMap<HookTrigger, Vec<Hook>> {
        let trigger: Vec<HookTrigger> = enum_iterator::all::<HookTrigger>().collect();
        let mut hooks: HashMap<HookTrigger, Vec<Hook>> = HashMap::new();
        for t in trigger {
            match t {
                HookTrigger::AgentSpawn => {
                    hooks.insert(
                        t,
                        self.agent_spawn
                            .iter()
                            .map(|h| Hook::from(h.clone()))
                            .collect(),
                    );
                }
                HookTrigger::UserPromptSubmit => {
                    hooks.insert(
                        t,
                        self.user_prompt_submit
                            .iter()
                            .map(|h| Hook::from(h.clone()))
                            .collect(),
                    );
                }
                HookTrigger::PreToolUse => {
                    hooks.insert(
                        t,
                        self.pre_tool_use
                            .iter()
                            .map(|h| Hook::from(h.clone()))
                            .collect(),
                    );
                }
                HookTrigger::PostToolUse => {
                    hooks.insert(
                        t,
                        self.post_tool_use
                            .iter()
                            .map(|h| Hook::from(h.clone()))
                            .collect(),
                    );
                }
                HookTrigger::Stop => {
                    hooks.insert(t, self.stop.iter().map(|h| Hook::from(h.clone())).collect());
                }
            };
        }
        hooks
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::Result, std::time::Duration};

    macro_rules! rando_hook {
        ($name:ident) => {
            impl $name {
                fn rando() -> $name {
                    let value = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    Self {
                        name: format!("$name-{value}"),
                        command: format!("{value}"),
                        timeout_ms: value,
                        max_output_size: 0,
                        cache_ttl_seconds: value,
                        matcher: Some(format!("{value}")),
                    }
                }
            }
        };
    }
    rando_hook!(HookAgentSpawn);
    rando_hook!(HookUserPromptSubmit);
    rando_hook!(HookPreToolUse);
    rando_hook!(HookPostToolUse);
    rando_hook!(HookStop);

    impl HookPart {
        pub fn randomize() -> Self {
            Self {
                agent_spawn: vec![HookAgentSpawn::rando()],
                user_prompt_submit: vec![HookUserPromptSubmit::rando()],
                pre_tool_use: vec![HookPreToolUse::rando()],
                post_tool_use: vec![HookPostToolUse::rando()],
                stop: vec![HookStop::rando()],
            }
        }
    }

    #[test_log::test]
    pub fn test_hooks_empty() -> Result<()> {
        let child = HookPart::default();
        let parent = HookPart::default();
        let merged = child.merge(parent);

        assert!(merged.agent_spawn.is_empty());
        assert!(merged.user_prompt_submit.is_empty());
        assert!(merged.pre_tool_use.is_empty());
        assert!(merged.post_tool_use.is_empty());
        assert!(merged.stop.is_empty());
        Ok(())
    }

    #[test_log::test]
    pub fn test_hooks_empty_child() -> Result<()> {
        let child = HookPart::default();
        let parent = HookPart::randomize();
        let before = parent.clone();
        let merged = child.merge(parent);

        assert_eq!(merged, before);
        Ok(())
    }

    #[test_log::test]
    pub fn test_hooks_no_merge() -> Result<()> {
        let child = HookPart::randomize();
        let parent = HookPart::randomize();
        let before = child.clone();
        let merged = child.merge(parent);
        assert_eq!(merged, before);
        Ok(())
    }

    #[test_log::test]
    pub fn test_hooks_merge_parent() -> Result<()> {
        let child = HookPart::randomize();
        std::thread::sleep(Duration::from_millis(1300)); // see randomize function
        let parent = HookPart::randomize();
        let merged = child.merge(parent);
        assert_eq!(merged.agent_spawn.len(), 2);
        assert_eq!(merged.user_prompt_submit.len(), 2);
        assert_eq!(merged.pre_tool_use.len(), 2);
        assert_eq!(merged.post_tool_use.len(), 2);
        assert_eq!(merged.stop.len(), 2);
        Ok(())
    }
}
