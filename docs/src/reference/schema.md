# Configuration Schema

kg provides JSON schemas for IDE autocompletion and validation.

## kg.toml Schema

For agent declarations in `kg.toml`:

```toml
"$schema" = "https://raw.githubusercontent.com/CarteraMesh/kiro-generator/refs/heads/main/schemas/kg.json"

[agents]
default = { inherits = [] }
```

## Agent Configuration Schema

For individual agent files like `default.toml`, `rust.toml`:

```toml
"$schema" = "https://raw.githubusercontent.com/CarteraMesh/kiro-generator/refs/heads/main/schemas/kiro-agent.json"

description = "Default agent"
allowedTools = ["read", "knowledge"]
```

## LSP Configuration

### taplo (Recommended)

Add to `.taplo.toml` in your project or `~/.config/taplo/.taplo.toml`:

```toml
[schema]
enabled = true

[[schema.associations]]
path = "**/kg.toml"
url = "https://raw.githubusercontent.com/CarteraMesh/kiro-generator/refs/heads/main/schemas/kg.json"

[[schema.associations]]
path = "**/.kiro/generators/*.toml"
url = "https://raw.githubusercontent.com/CarteraMesh/kiro-generator/refs/heads/main/schemas/kiro-agent.json"
```

### VS Code

Install the [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) extension. It uses taplo and will automatically pick up the `$schema` field from your TOML files.

### Neovim

Using `nvim-lspconfig`:

```lua
require('lspconfig').taplo.setup({
  settings = {
    evenBetterToml = {
      schema = {
        enabled = true,
        associations = {
          ["**/kg.toml"] = "https://raw.githubusercontent.com/CarteraMesh/kiro-generator/refs/heads/main/schemas/kg.json",
          ["**/.kiro/generators/*.toml"] = "https://raw.githubusercontent.com/CarteraMesh/kiro-generator/refs/heads/main/schemas/kiro-agent.json"
        }
      }
    }
  }
})
```

## Benefits

With schema validation you get:

- **Autocompletion** - Field suggestions as you type
- **Validation** - Immediate feedback on typos and invalid values
- **Documentation** - Hover tooltips explaining each field
- **Type checking** - Catch errors before running `kg validate`

## Schema Location

The schema is versioned with releases:

- **Latest:** `https://raw.githubusercontent.com/CarteraMesh/kiro-generator/refs/heads/main/schemas/kg.json`
- **Specific version:** `https://raw.githubusercontent.com/CarteraMesh/kiro-generator/refs/tags/v0.1.0/schemas/kg.json`

Pin to a specific version for stability or use `main` for latest features.
