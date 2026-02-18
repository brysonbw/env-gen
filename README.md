# Env Generator

A command line tool for generating an environment variable file.

[![Crates.io](https://img.shields.io/crates/v/env-gen?style=flat)](https://crates.io/crates/env-gen)
[![Crates.io](https://img.shields.io/crates/d/env-gen?style=flat)](https://crates.io/crates/env-gen)
![CI](https://img.shields.io/github/actions/workflow/status/brysonbw/env-gen/ci.yml?branch=main&style=flat&logo=github&label=CI)

## Install

```bash
cargo install env-gen
```

## Usage

```bash
env-gen
```

> This will prompt you to create a new configuration file, optionally using an existing template file, and add key-value pairs for your environment variables. The generated `.env` file will be created in the current working directory.

> The following template files are supported (searched in the current working > directory):
>
> - `example.env`
> - `.example.env`
> - `env.example`
> - `.env.example`
> - `env.sample`
> - `.env.sample`
> - `.env-dist`
