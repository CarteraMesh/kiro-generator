# Config Files

kg uses a hierarchical TOML-based configuration system. Configuration can be defined globally, locally, or both.

## File Locations

Configuration is loaded in order from least to most precedence:

1. `~/.kiro/generators/[agent-name].toml` - Global agent config
2. `~/.kiro/generators/kg.toml` - Global agent declarations
3. `.kiro/generators/[agent-name].toml` - Local agent config
4. `.kiro/generators/kg.toml` - Local agent declarations

Local settings override global settings. Both are merged together unless you use `--local` to ignore global config.

## File Types

### kg.toml

The main configuration file that declares agents and their relationships.

```toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
```

### agent-name.toml

Individual agent configuration files containing the actual settings.

```toml
description = "Default agent"
tools = ["*"]
allowedTools = ["read", "knowledge", "web_search"]
resources = ["file://README.md"]

[toolsSettings.shell]
allowedCommands = ["git status", "git fetch"]
autoAllowReadonly = true
```

## Inline Configuration

You can define agent properties directly in `kg.toml`:

```toml
[agents.default]
inherits = []
allowedTools = ["read", "knowledge"]
resources = ["file://README.md"]

[agents.rust]
inherits = ["default"]
allowedTools = ["@rustdocs", "@cargo"]
```

This is useful for simple agents or when you want everything in one file.
