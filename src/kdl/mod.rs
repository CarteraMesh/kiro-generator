mod agent;
mod hook;
mod mcp;
mod native;

pub use agent::KdlAgent;

#[derive(knuffel::Decode)]
pub struct GeneratorConfig {
    #[knuffel(children(name = "agent"))]
    pub agents: Vec<KdlAgent>,
}

#[cfg(test)]
mod tests {
    use {super::*, crate::agent::hook::HookTrigger, color_eyre::eyre::eyre, knuffel::parse};

    #[test_log::test]
    fn test_agent_decoding() -> crate::Result<()> {
        let kdl_agents = r#"
            agent "test" {
                inherits "parent"
                description "This is a test agent"
                prompt "Generate a test prompt"
                resource "file://resource.md"
                resource "file://README.md"
                include-mcp-json
                tools "*"

                allowed-tools "@awsdocs"
                hook {
                    agent-spawn {
                        command "echo i have spawned"
                        timeout-ms 1000
                        max-output-size 9000
                        cache-ttl-seconds 2
                    }
                    user-prompt-submit {
                        command "echo user submitted"
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

                mcp "awsdocs" {
                   command "aws-docs"
                   args "--verbose" "--config=/path"
                   env "RUST_LOG" "debug"
                   env "PATH" "/usr/bin"
                   header "Authorization" "Bearer token"
                   timeout 5000
                   oauth {
                       redirect-uri "127.0.0.1:7778"
                   }
                }

                alias "execute_bash" "shell"

                native {
                   write {
                       allow "./src/*" "./scripts/**"
                       deny  "Cargo.lock"
                       force "/tmp"
                       force "/var/log"
                   }
                   shell deny-by-default=true {
                      allow "git status .*"
                      deny "git push .*"
                      force "git pull .*"
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
        assert_eq!(config.agents.len(), 1);
        let agent = config.agents[0].clone();
        assert_eq!(agent.name, "test");
        assert!(agent.model.is_none());
        assert!(!agent.skeleton);
        let inherits = agent.inherits();
        assert_eq!(inherits.len(), 1);
        assert_eq!(inherits.iter().next().unwrap(), "parent");
        assert!(agent.description.is_some());
        assert!(agent.prompt.is_some());
        assert!(agent.include_mcp_json);
        let tools = agent.tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools.iter().next().unwrap(), "*");
        let resources = agent.resources();
        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0], "file://resource.md");
        assert_eq!(resources[1], "file://README.md");

        let hook = agent.hook(HookTrigger::AgentSpawn);
        assert!(hook.is_some());
        let hook = hook.unwrap();
        assert_eq!(hook.command, "echo i have spawned");

        assert!(agent.hook(HookTrigger::PreToolUse).is_some());
        assert!(agent.hook(HookTrigger::PostToolUse).is_some());
        assert!(agent.hook(HookTrigger::Stop).is_some());
        assert!(agent.hook(HookTrigger::UserPromptSubmit).is_some());

        let allowed = agent.allowed_tools();
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed.iter().next().unwrap(), "@awsdocs");

        let mcp = agent.mcp_servers();
        assert_eq!(mcp.len(), 1);
        assert!(mcp.contains_key("awsdocs"));
        let aws_docs = mcp.get("awsdocs").unwrap();
        assert_eq!(aws_docs.command, "aws-docs");
        assert_eq!(aws_docs.args, vec!["--verbose", "--config=/path"]);
        assert!(!aws_docs.disabled);
        assert_eq!(aws_docs.headers.len(), 1);
        assert_eq!(aws_docs.env.len(), 2);
        assert_eq!(aws_docs.timeout, 5000);
        assert!(aws_docs.oauth.is_some());

        assert_eq!(agent.tool_aliases().len(), 1);
        Ok(())
    }
}
