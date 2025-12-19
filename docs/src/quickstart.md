# Quick Start 

1. Define your agents in `~/.kiro/generators/kg.toml`
  `cat ~/.kiro/generators/kg.toml`

  ```toml
  [agents]
  default = {  } # default is the agent name
  rust = { inherits = ["default"] }  # rust agent config is merged with default
  ```

2. Define your agent configurations in `~/.kiro/generators/<agent name>.toml`

  `cat ~/.kiro/generators/default.toml`
  ```toml
  description = "Default agent"
  tools = ["*"]
  allowedTools = ["read", "knowledge", "web_search"]
  resources = ["file://README.md", "file://AGENTS.md"]
  [toolsSettings.shell]
  allowedCommands = ["git status", "git fetch", "git diff .*" ]
  deniedCommands = ["git commit .*", "git push .*" ]
  autoAllowReadonly = true
  ```

  `cat ~/.kiro/generators/rust.toml`
  
  ```toml
  description = "General Rust agent"
  resources = ["file://RUST.md"]
  allowedTools = [ "@rustdocs", "@cargo" ] # also ["read", "knowledge", "web_search"] from default.toml
  [mcpServers]
  rustdocs = { type = "stdio" , command = "rust-docs-mcp", timeout = 1000 }
  cargo = {  command = "cargo-mcp" , timeout = 1200  }
  
  [toolsSettings]
  [toolsSettings.shell]
  allowedCommands = ["cargo .+" ] ## inherits allowedCommands from default.toml
  deniedCommands = ["cargo publish .*"] ## inherits allowedCommands from default.toml
  ```

3. Validate
  ```shell
  $ kg validate 
  ╭────────────────────┬─────┬─────────────────┬────────────────────────────────────────────────┬────────────────────┬────────┬────────┬────────╮
  │ Agent 🤖 (PREVIEW) ┆ Loc ┆ MCP 💻          ┆ Allowed Tools ⚙️                               ┆ Resources 📋       ┆    Forced Permissions    │
  ╞════════════════════╪═════╪═════════════════╪════════════════════════════════════════════════╪════════════════════╪══════════════════════════╡
  │ default            ┆ 📁  ┆                 ┆ knowledge, read, web_search                    ┆ - file://README.md ┆                          │
  │                    ┆     ┆                 ┆                                                ┆ - file://AGENTS.md ┆                          │
  │                    ┆     ┆                 ┆                                                ┆                    ┆                          │
  ├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
  │ rust               ┆ 📁  ┆ cargo, rustdocs ┆ @cargo, @rustdocs, knowledge, read, web_search ┆ - file://README.md ┆                          │
  │                    ┆     ┆                 ┆                                                ┆ - file://AGENTS.md ┆                          │
  │                    ┆     ┆                 ┆                                                ┆ - file://RUST.md   ┆                          │
  │                    ┆     ┆                 ┆                                                ┆                    ┆                          │
  ╰────────────────────┴─────┴─────────────────┴────────────────────────────────────────────────┴────────────────────┴──────────────────────────╯
  
  🎉 Config is valid
  → Run kg generate to generate agent files
  ```
