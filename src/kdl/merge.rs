use super::*;

impl KdlAgent {
    pub(super) fn merge(mut self, other: KdlAgent) -> Self {
        self.include_mcp_json |= other.include_mcp_json;
        self.resources.extend(other.resources);
        self.tools.tools.extend(other.tools.tools);
        self.allowed_tools
            .allowed
            .extend(other.allowed_tools.allowed);
        if self.description.is_none()
            && let Some(d) = other.description
        {
            self.description = Some(d);
        }
        if self.prompt.is_none()
            && let Some(p) = other.prompt
        {
            self.prompt = Some(p);
        }

        if self.model.is_none()
            && let Some(m) = other.model
        {
            self.model = Some(m);
        }

        match (&self.hook, other.hook) {
            (None, Some(h)) => self.hook = Some(h),
            (Some(a), Some(b)) => self.hook = Some(a.clone().merge(b)),
            _ => {}
        };

        self.tool_aliases.extend(other.tool_aliases);
        self
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::agent::hook::HookTrigger, color_eyre::eyre::eyre, knuffel::parse};
    #[test_log::test]
    fn test_agent_merge() -> crate::Result<()> {
        let kdl_agents = r#"
            agent "child" {
               description "I am a child"
               resource "file://child.md"
               resource "file://README.md"
               inherits "parent"
               include-mcp-json
               tools "@awsdocs" "shell"
               hook {
                 agent-spawn {
                     command "echo i have spawned"
                     max-output-size 9000
                     cache-ttl-seconds 2
                 }
               }
                alias "execute_bash" "shell"
            }
            agent "parent" {
               description "I am parent"
               skeleton
               resource "file://parent.md"
               resource "file://README.md"
               tools "web_search" "shell"
               prompt "i tell you what to do"
               model "claude"
               allowed-tools "write"
               alias "execute_bash" "shell"
               alias "fs_read" "read"
               hook {
                   agent-spawn {
                     timeout-ms 1111
                   }
                   user-prompt-submit {
                       command "echo user submitted"
                       timeout-ms 1000
                   }
                   pre-tool-use {
                       command "echo before tool"
                       matcher "git.*"
                   }
                   post-tool-use {
                       command "echo after tool"
                   }
                   stop {
                       command "echo stopped"
                   }
               }
            }
        "#;

        let config: GeneratorConfig = match parse("example.kdl", kdl_agents) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{:?}", miette::Report::new(e));
                return Err(eyre!("failed to parse {kdl_agents}"));
            }
        };
        assert_eq!(config.agents.len(), 2);
        let child = config
            .agents
            .iter()
            .find(|a| a.name == "child")
            .unwrap()
            .clone();
        let parent = config
            .agents
            .iter()
            .find(|a| a.name == "parent")
            .unwrap()
            .clone();
        let merged = child.merge(parent);
        assert!(merged.description.is_some());
        let d = merged.description.clone().unwrap();
        assert_eq!(d, "I am a child");

        assert_eq!(merged.resources.len(), 3);
        assert!(!merged.skeleton);
        assert!(merged.include_mcp_json);

        assert_eq!(merged.inherits.parents.len(), 1);
        assert!(merged.inherits.parents.contains("parent"));

        assert_eq!(merged.prompt, Some("i tell you what to do".to_string()));
        let tools = merged.tools();
        assert_eq!(tools.len(), 3);
        assert!(tools.contains("@awsdocs"));
        assert!(tools.contains("shell"));
        assert!(tools.contains("web_search"));

        assert_eq!(merged.model, Some("claude".to_string()));

        let allowed_tools = merged.allowed_tools();
        assert_eq!(allowed_tools.len(), 1);
        assert!(allowed_tools.contains("write"));

        let h = merged.hook(HookTrigger::AgentSpawn);
        assert!(h.is_some());
        let h = h.unwrap();
        assert_eq!(h.timeout_ms, 1111);
        assert_eq!(h.command, "echo i have spawned");

        let h = merged.hook(HookTrigger::UserPromptSubmit);
        assert!(h.is_some());
        let h = h.unwrap();
        assert_eq!(h.command, "echo user submitted");
        assert_eq!(h.timeout_ms, 1000);

        let alias = merged.tool_aliases();
        assert_eq!(alias.len(), 2);
        assert!(alias.contains_key("fs_read"));
        assert!(alias.contains_key("execute_bash"));
        Ok(())
    }
}
