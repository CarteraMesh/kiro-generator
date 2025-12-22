use {
    crate::agent::hook::{Hook, HookTrigger},
    knuffel::Decode,
};

macro_rules! define_hook {
    ($name:ident) => {
        #[derive(Decode, Clone, Debug)]
        pub(super) struct $name {
            /// The command to run when the hook is triggered
            #[knuffel(child, unwrap(argument))]
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
