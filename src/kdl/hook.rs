use {
    crate::agent::hook::{Hook, HookTrigger},
    knuffel::Decode,
};

macro_rules! define_hook {
    ($name:ident) => {
        #[derive(Decode, Default, Clone, Debug)]
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
    #[knuffel(child, default)]
    pub agent_spawn: HookAgentSpawn,
    #[knuffel(child, default)]
    pub user_prompt_submit: HookUserPromptSubmit,
    #[knuffel(child, default)]
    pub pre_tool_use: HookPreToolUse,
    #[knuffel(child, default)]
    pub post_tool_use: HookPostToolUse,
    #[knuffel(child, default)]
    pub stop: HookStop,
}

impl HookPart {
    pub fn merge(mut self, other: Self) -> Self {
        self.agent_spawn = self.agent_spawn.merge(other.agent_spawn);
        self.user_prompt_submit = self.user_prompt_submit.merge(other.user_prompt_submit);
        self.pre_tool_use = self.pre_tool_use.merge(other.pre_tool_use);
        self.post_tool_use = self.post_tool_use.merge(other.post_tool_use);
        self.stop = self.stop.merge(other.stop);
        self
    }

    pub fn get(&self, trigger: HookTrigger) -> Option<Hook> {
        match trigger {
            HookTrigger::AgentSpawn => match self.agent_spawn.command.is_empty() {
                true => None,
                false => Some(Hook::from(self.agent_spawn.clone())),
            },
            HookTrigger::UserPromptSubmit => match self.user_prompt_submit.command.is_empty() {
                true => None,
                false => Some(Hook::from(self.user_prompt_submit.clone())),
            },
            HookTrigger::PreToolUse => match self.pre_tool_use.command.is_empty() {
                true => None,
                false => Some(Hook::from(self.pre_tool_use.clone())),
            },
            HookTrigger::PostToolUse => match self.post_tool_use.command.is_empty() {
                true => None,
                false => Some(Hook::from(self.post_tool_use.clone())),
            },
            HookTrigger::Stop => match self.stop.command.is_empty() {
                true => None,
                false => Some(Hook::from(self.stop.clone())),
            },
        }
    }
}
