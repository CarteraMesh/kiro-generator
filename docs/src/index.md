## About

`kiro-generator` (kg) is a CLI tool for managing and generating [Kiro](https://kiro.dev/docs/) agent files.

## Why?

Because managing config files via `JSON` is the second worse format. Obviously `YAML` files takes 1st prize

## Prerequisites

* [kiro-cli](https://kiro.dev/cli/)
* A distaste for `JSON` config files

## Features

### Config Hierarchy 

KG provides a hierarchical configuration system that allows you to define and override settings at different levels. The schemas is mostly the same as `kiro-cli` JSON's format but defined as TOML, with a few added fields explained in [usage](./usage.md)

### Agent Declaration

By default Agents can be declared globally `~/.kiro/generators/kg.toml` or locally `.kiro/generators/kg.toml`. If both are present, the local configuration takes precedence, however both configurations are merged together. You can use `--local` argument to ignore global configuration.


### Force Permissions 

You can override toolsettings permissions. For example, you can have your `default` agent deny executing `git push`, but override this for special use cases, see [inheritance](config/inheritance.md) and [usage](./usage.md) for more information.
