# Agent Declaration

Agents are declared in [kg.toml](./files.md) files using the `[agents]` table.

## Basic Declaration

```toml
[agents]
default = { inherits = [] }
```

The key (`default`) is the agent name. This creates an agent that looks for configuration in:
- `~/.kiro/generators/default.toml`
- `.kiro/generators/default.toml`

## Agent Properties

### inherits

List of parent agents to inherit configuration from.

```toml
[agents]
default = { inherits = [] }
rust = { inherits = ["default"] }
python = { inherits = ["default"] }
```

### skeleton

Mark an agent as a template that won't generate a JSON file.

```toml
[agents]
git-base = { skeleton = true }
```

See [Skeletons](./skeletons.md) for details.

## Inline vs External Config

You can define agent configuration inline or in separate files:

**Inline:**
```toml
[agents.default]
inherits = []
allowedTools = ["read", "knowledge"]
```

**External:**
```toml
[agents]
default = { inherits = [] }
```

Then create `~/.kiro/generators/default.toml`:
```toml
allowedTools = ["read", "knowledge"]
```

Both approaches can be mixed. Inline config takes precedence.

## Location

Agents can be declared globally or locally:

- **Global:** `~/.kiro/generators/kg.toml` - Available in all projects
- **Local:** `.kiro/generators/kg.toml` - Project-specific agents

Use `kg validate` to see the merged configuration.
