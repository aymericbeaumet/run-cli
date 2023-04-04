# workbench [![GitHub Actions](https://github.com/aymericbeaumet/workbench/actions/workflows/ci.yml/badge.svg)](https://github.com/aymericbeaumet/workbench/actions/workflows/ci.yml)

[workbench](https://github.com/aymericbeaumet/workbench) is a tool allowing to easily manage processes you have to run when working on a project. For example to work on your app you need to: start the frontend, start the backend, start the database, run the migrations, etc. This is done via a simple `workbench.toml` file. See the [examples](./examples) to understand how you could integrate it in your workflow.

## Features

- Works on Linux, macOS and Windows
- Forwards all the signals to the child process
- Watch mode to restart process when dependencies change
- Can run processes sequentially, concurrently, or in tmux

## Install

### Using git

_This method requires the [Rust
toolchain](https://www.rust-lang.org/tools/install) to be installed on your
machine._

```
git clone -–depth=1 https://github.com/aymericbeaumet/workbench.git /tmp/workbench
cargo install --path=/tmp/workbench/workbench-cli
```
