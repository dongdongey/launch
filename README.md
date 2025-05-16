# Program Launcher

This is a simple launcher I built because running multiple servers manually was getting ridiculously annoying.

## 🚀 Features

-   Run multiple background or instant tasks from a single config file
-   TOML-based configuration
-   Easy start/restart/stop commands
-   Per-task working directory, logging, and env support

## Usage

By default, this program looks for a `Launch.toml` file in your home directory.

### Example

```toml
# Launch.toml

[[task_name]] # A background task that runs independently from the current session
command = "command1 ar gu ment"
current_dir = "/current/working/directory1"
log_file = "/your/log/file1.log"

[[task_name]] # You can run multiple background processes simultaneously
command = "command2 ar gu ment"
current_dir = "/current/working/directory1"
log_file = "/your/log/file2.log"
env = { KEY1 = "value1", KEY2 = "value2" }

[instant_task] # A task that runs and exits within the current session
command = "cargo run -r"
current_dir = "/my/cargo/project/directoy"

[[another_task]] # You can define multiple background tasks
command = "another_server"
current_dir = "/another/server/working/directory"

[another_instant_task] # And multiple instant tasks too
command = "node ./"
current_dir = "/my/nodejs/project/directoy"
```

-   Tasks that run **in the current session** use **single square brackets**: `[name]`
-   Tasks that run **in the background** use **double square brackets**: `[[name]]`

The `command` field is required.
`current_dir`, `log_file`, and `env` are optional.

---

## Commands

### Run a background task

```
launch < start | end | restart > <task_name> [options...]

Example: launch start task_name
```

### Run an instant (foreground) task

```
launch <task_name> [options...]

Example: launch instant_task
```

### List all tasks

```
launch list
```

### Options

```
--path | -P <path> : Run launch with a specific directory or file

Example: launch end another_task --path ./
(looks for a Launch.toml file in the current directory)
```

---

**📝 Be careful when editing `Launch.toml`!**

While tasks are running, the `Launch.toml` file includes their `pid` information.
If you manually edit the file during execution, the `pid` data may become inconsistent,
which can break `end` or `restart` commands.

> **In short: avoid editing `Launch.toml` while tasks are running.**
