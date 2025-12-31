use super::*;

impl KdlAgent {
    pub fn merge(mut self, other: KdlAgent) -> Self {
        // Child wins for explicit values
        self.include_mcp_json = self.include_mcp_json.or(other.include_mcp_json);
        self.template = self.template.or(other.template);
        self.description = self.description.or(other.description);
        self.prompt = self.prompt.or(other.prompt);
        self.model = self.model.or(other.model);

        // Collections are extended (merged)
        self.resources.extend(other.resources);
        self.tools.extend(other.tools);
        self.allowed_tools.extend(other.allowed_tools);
        self.alias.extend(other.alias);
        self.mcp.extend(other.mcp);
        self.inherits.extend(other.inherits);
        self.tool_setting.extend(other.tool_setting);

        self.hook = self.hook.merge(other.hook);
        self.native_tool = self.native_tool.merge(other.native_tool);

        self
    }
}

// #[cfg(test)]
// mod tests {
//     use {super::*, crate::agent::hook::HookTrigger, color_eyre::eyre::eyre,
// knuffel::parse};

//     #[test_log::test]
//     fn test_agent_merge() -> crate::Result<()> {
//         let kdl_agents = r#"
//             agent "child" template=$false {
//                description "I am a child"
//                resource "file://child.md"
//                resource "file://README.md"
//                inherits "parent"
//                include-mcp-json true
//                tools "@awsdocs" "shell"
//                native-tool {
//                   write {
//                     override  "Cargo.lock"
//                   }
//                   shell {
//                     override  "git push .*"
//                   }
//                }
//                hook {
//                  agent-spawn "spawn" {
//                      command "echo i have spawned"
//                      max-output-size 9000
//                      cache-ttl-seconds 2
//                  }
//                }
//                 alias "execute_bash" "shell"
//             }
//             agent "parent" template=#true {
//                description "I am parent"
//                resource "file://parent.md"
//                resource "file://README.md"
//                tools "web_search" "shell"
//                prompt "i tell you what to do"
//                model "claude"
//                allowed-tools "write"
//                alias "execute_bash" "shell"
//                alias "fs_read" "read"
//                native-tool {
//                  read {
//                      allow "./src/*" "./scripts/**"
//                      deny  "Cargo.lock"
//                  }
//                   write {
//                       allow "./src/*" "./scripts/**"
//                       deny  "Cargo.lock"
//                   }

//                   shell {
//                       allow "git status .*" "git pull .*"
//                       deny  "git push .*"
//                   }
//                }
//                hook {
//                    agent-spawn "spawn" {
//                      timeout-ms 1111
//                    }
//                    user-prompt-submit "submit" {
//                        command "echo user submitted"
//                        timeout-ms 1000
//                    }
//                    pre-tool-use "pre" {
//                        command "echo before tool"
//                        matcher "git.*"
//                    }
//                    post-tool-use "post" {
//                        command "echo after tool"
//                    }
//                    stop "stop" {
//                        command "echo stopped"
//                    }
//                }
//             }
//         "#;

//         let config: GeneratorConfig = match parse("example.kdl", kdl_agents)
// {             Ok(c) => c,
//             Err(e) => {
//                 eprintln!("{:?}", miette::Report::new(e));
//                 return Err(eyre!("failed to parse {kdl_agents}"));
//             }
//         };
//         assert_eq!(config.agents.len(), 2);
//         let child = config
//             .agents
//             .iter()
//             .find(|a| a.name == "child")
//             .unwrap()
//             .clone();
//         let parent = config
//             .agents
//             .iter()
//             .find(|a| a.name == "parent")
//             .unwrap()
//             .clone();
//         let merged = child.merge(parent);
//         assert!(merged.description.is_some());
//         let d = merged.description.clone().unwrap();
//         assert_eq!(d, "I am a child");

//         assert_eq!(merged.resources.len(), 3);
//         assert!(!merged.is_template());
//         assert!(merged.include_mcp_json());

//         assert_eq!(merged.inherits.parents.len(), 1);
//         assert!(merged.inherits.parents.contains("parent"));

//         assert_eq!(merged.prompt, Some("i tell you what to do".to_string()));
//         let tools = merged.tools();
//         assert_eq!(tools.len(), 3);
//         assert!(tools.contains("@awsdocs"));
//         assert!(tools.contains("shell"));
//         assert!(tools.contains("web_search"));

//         assert_eq!(merged.model, Some("claude".to_string()));

//         let allowed_tools = merged.allowed_tools();
//         assert_eq!(allowed_tools.len(), 1);
//         assert!(allowed_tools.contains("write"));

//         let hooks = merged.hooks();
//         assert!(!hooks.is_empty());
//         let h = hooks.get(&HookTrigger::AgentSpawn);
//         assert!(h.is_some());
//         let h = h.unwrap();
//         assert!(!h.is_empty());
//         assert_eq!(h[0].timeout_ms, 1111);
//         assert_eq!(h[0].command, "echo i have spawned");

//         let h = hooks.get(&HookTrigger::UserPromptSubmit);
//         assert!(h.is_some());
//         let h = h.unwrap();
//         assert!(!h.is_empty());
//         assert_eq!(h[0].command, "echo user submitted");
//         assert_eq!(h[0].timeout_ms, 1000);

//         let alias = merged.tool_aliases();
//         assert_eq!(alias.len(), 2);
//         assert!(alias.contains_key("fs_read"));
//         assert!(alias.contains_key("execute_bash"));

//         let tool = merged.get_tool_write();
//         assert!(tool.override_path.contains(&"Cargo.lock".into()));
//         assert_eq!(tool.allow.list.len(), 2);
//         assert_eq!(tool.override_path.len(), 1);
//         assert_eq!(tool.deny.list.len(), 1);

//         let tool = merged.get_tool_read();
//         assert_eq!(tool.allow.list.len(), 2);
//         assert_eq!(tool.override_path.len(), 0);
//         assert_eq!(tool.deny.list.len(), 1);

//         let tool = merged.get_tool_shell();
//         assert_eq!(tool.allow.list.len(), 2);
//         assert_eq!(tool.override_command.len(), 1);
//         assert_eq!(tool.deny.list.len(), 1);

//         let tool = merged.get_tool_aws();
//         assert!(tool.allow.list.is_empty());
//         assert!(tool.deny.list.is_empty());

//         assert_eq!("child", format!("{merged}"));
//         assert_eq!("child", format!("{merged:?}"));
//         Ok(())
//     }
// }
