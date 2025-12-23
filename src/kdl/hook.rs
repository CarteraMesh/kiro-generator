use {
    crate::agent::hook::{Hook, HookTrigger},
    knuffel::Decode,
};

macro_rules! define_hook {
    ($name:ident) => {
        #[derive(Decode, Clone, Debug)]
        pub(super) struct $name {
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

#[derive(Decode, Clone, Default, Debug)]
pub(super) struct HookPart {
    #[knuffel(child)]
    pub agent_spawn: Option<HookAgentSpawn>,
    #[knuffel(child)]
    pub user_prompt_submit: Option<HookUserPromptSubmit>,
    #[knuffel(child)]
    pub pre_tool_use: Option<HookPreToolUse>,
    #[knuffel(child)]
    pub post_tool_use: Option<HookPostToolUse>,
    #[knuffel(child)]
    pub stop: Option<HookStop>,
}

impl HookPart {
    pub fn merge(mut self, other: Self) -> Self {
        match (&self.agent_spawn, &other.agent_spawn) {
            (None, None) | (Some(_), None) => {}
            (None, Some(o)) => self.agent_spawn = Some(o.clone()),
            (Some(a), Some(o)) => {
                let merged = a.clone();
                let other = o.clone();
                self.agent_spawn = Some(merged.merge(other));
            }
        };

        match (&self.user_prompt_submit, &other.user_prompt_submit) {
            (None, None) | (Some(_), None) => {}
            (None, Some(o)) => self.user_prompt_submit = Some(o.clone()),
            (Some(a), Some(o)) => {
                let merged = a.clone();
                let other = o.clone();
                self.user_prompt_submit = Some(merged.merge(other));
            }
        };

        match (&self.post_tool_use, &other.post_tool_use) {
            (None, None) | (Some(_), None) => {}
            (None, Some(o)) => self.post_tool_use = Some(o.clone()),
            (Some(a), Some(o)) => {
                let merged = a.clone();
                let other = o.clone();
                self.post_tool_use = Some(merged.merge(other));
            }
        };

        match (&self.pre_tool_use, &other.pre_tool_use) {
            (None, None) | (Some(_), None) => {}
            (None, Some(o)) => self.pre_tool_use = Some(o.clone()),
            (Some(a), Some(o)) => {
                let merged = a.clone();
                let other = o.clone();
                self.pre_tool_use = Some(merged.merge(other));
            }
        };

        match (&self.stop, &other.stop) {
            (None, None) | (Some(_), None) => {}
            (None, Some(o)) => self.stop = Some(o.clone()),
            (Some(a), Some(o)) => {
                let merged = a.clone();
                let other = o.clone();
                self.stop = Some(merged.merge(other));
            }
        };
        self
    }

    pub fn get(&self, trigger: HookTrigger) -> Option<Hook> {
        match trigger {
            HookTrigger::AgentSpawn => self.agent_spawn.clone().map(Into::into),
            HookTrigger::UserPromptSubmit => self.user_prompt_submit.clone().map(Into::into),
            HookTrigger::PreToolUse => self.pre_tool_use.clone().map(Into::into),
            HookTrigger::PostToolUse => self.post_tool_use.clone().map(Into::into),
            HookTrigger::Stop => self.stop.clone().map(Into::into),
        }
    }
}
