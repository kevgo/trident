# Trident

[![linux](https://github.com/kevgo/Trident/actions/workflows/linux.yml/badge.svg)](https://github.com/kevgo/trident/actions/workflows/linux.yml)
[![windows](https://github.com/kevgo/Trident/actions/workflows/windows.yml/badge.svg)](https://github.com/kevgo/trident/actions/workflows/windows.yml)

Trident determines and runs all applicable automated software quality
guardrails: type checkers, linters, formatters, fixes, unit tests, etc.
For all file types in your codebase, concurrently for fast results.
This is useful for AI engineering and manual coding.

## Demo

If I run `trident lint --scope=all --show=output` on the Trident codebase,
using a 2021 MacBook Pro M1 running Linux,
it finishes in about 0.2 seconds (!) and prints:

```sh
163 Cucumber, 4 JSON, 1 JSONC, 4 Markdown, 127 Rust, 4 TOML, 4 YML, 8 other
running 7 tools

lint Markdown (rumdl)
lint TOML (Taplo)
lint Git diff markers (git diff HEAD --check)
GitHub Actions (actionlint)
lint Rust (cargo clippy for Cucumber code)
lint Rust (cargo clippy)
lint Cucumber (gherkin-lint)
```

Trident has analyzed 315 files and executed 7 linters concurrently.
You don't need to know about or even install these third-party tools,
Trident [downloads and runs them](https://github.com/kevgo/run-that-app)
as needed.

## Why is it so fast

Trident is designed from the ground up for speed.

- It favors modern linters and formatters that execute quickly.
- It runs all tools concurrently, even tools that change files!
- It passes each tool the exact files to process,
  so that tools don't need to discover files again.
- It executes the tools without any emulation or container layer.
- It is written in Rust.

## Usage

Trident supports three categories of tools, running in this order:

1. **Fixes** correct code smells.
   They don't need to worry about formatting,
   Trident runs the formatter afterwards.
2. **Formatters** improve how code is formatted.
3. **Checks** look for all remaining not auto-fixable code smells
   and run unit tests.

Trident provides special commands
for all relevant phases of the software development workflow:

### `trident pitstop`

Run this command at any time during human development.
It first applies all safe fixes, then formats everything,
then reports any remaining code problems that require (manual or AI) attention.

This command reviews the scope you currently work on:

- if uncommitted changes exist: it reviews only those
- without uncommitted changes but the feature branch you are on has changes:
  it reviews all changes made on that branch
- otherwise: it reviews the entire repo

You can configure which changes get processed with the `--scope` CLI flag.

You can also configure it to run certain global tests during a pitstop:

```jsonc
{
  "commands": {
    "pitstop": {
      "test": ["unit"]
    }
  }
}
```

### `trident postedit`

This command is the equivalent of `trident pitstop` for AI agents.
Each time your agent generates code, this command checks it for code smells.
It does not format files because coding agents cache file contents
and can get tripped up by unexpected file changes.
In the AI workflow, the changes get formatted when being committed.

### `trident precommit`

This command runs inside the Git precommit hook.
It formats all staged files.
It always exits with status code 0, so the commit always goes through,
even if the staged changes cannot be formatted.

This command stages files
that got formatted to make sure the formatting changes get committed too.

> [!TIP]
> Re-staging operates on the entire file.
> If you want to commit only part of a modified file,
> run [Trident pitstop](#trident-pitstop)
> or [Trident fix](#trident-fix) to format everything before partially staging
> and committing your changes.
> That way, Trident precommit won't introduce additional formatting changes
> and won't need to re-stage the file.

### `trident ci`

This command runs inside your CI pipeline.
It ensures everything is green, i.e. it fails if either:

- some code is unformatted, i.e. a formatter or fix would modify a file
- a check reports an unresolved issue
- any of the global tests defined in `trident.jsonc` fails

To run only some tests, configure them in the config file
or pass their names on the CLI, joined with `+`:

```sh
trident ci --test=unit+e2e
```

### `trident init:config`

Creates an Trident config file with the default settings.

### `trident init:claude`

This command integrates Trident into coding agents harnesses
that use the Claude Code configuration format, such as Claude Code, Codex,
Code Puppy, or Wibey.

Once configured, your AI agent will run `trident postedit`
after it makes file changes.

### `trident init:githook`

This command installs a
[Git pre-commit hook](https://git-scm.com/book/ms/v2/Customizing-Git-Git-Hooks)
that runs `trident precommit` before every commit.

### `trident update:tools`

This command updates the versions of third-party tools that Trident runs
for you to the latest available versions.
These changes get written to a file with the name `run-that-app`, which uses the
[asdf config syntax](https://asdf-vm.com/manage/configuration.html).

### `trident fix`

This command applies all safe automated fixes to your current work.
Fixes for different file types are processed concurrently,
multiple fixes for the same file type run sequentially.

### `trident fix-unsafe`

This command applies more aggressive automatic fixes to your current work.
These might change program behavior, please verify the results.

### `trident lint`

This command runs all linters that apply to your current work.
All linters run in parallel.

### `trident test`

This command runs all tests defined in `trident.jsonc` in parallel.

To run only some tests, configure `commands.test.test` in the config file
or pass their names joined with `+`:

```sh
trident test --test=unit+e2e
```

### `trident full`

This command runs everything: all safe fixes, all lints, all tests,
on all files.

### `--scope`

`ci`, `fix`, `fix-unsafe`, `lint`,
and `pitstop` accept `--scope` to choose which files they process:

- `uncommitted`: only uncommitted files (staged, unstaged, and untracked)
- `branch`: only files that the current branch changes
- `all`: all files in the current directory

The `ci` command defaults to `all` files.
The `fix`, `fix-unsafe`, `lint`,
and `pitstop` commands choose a scope matching the current situation:

- uncommitted changes present: process only those
- no uncommitted changes but on a feature branch:
  process all files on the current branch
- otherwise process all files

### `--sequential`

By default, Trident runs as many tools as possible concurrently.
The `--sequential` flag makes it run all tools one after another instead.
This is slower, but helps when tools interfere with each other
or the machine cannot handle the load of running them all at once.

```sh
trident ci --sequential
```

## Supported stacks

<!-- SUPPORTED-STACKS-START -->

| Stack      | Linter                | Fix                   |
| ---------- | --------------------- | --------------------- |
| CSS        | Biome                 | Biome                 |
| Cucumber   | gherkin-lint          | cucumber-sort, Ghokin |
| Dockerfile | hadolint              |                       |
| Go         | golangci-lint         | gofumpt               |
| JSON       | Biome                 | Biome                 |
| JSONC      | Biome                 | Biome                 |
| Markdown   | rumdl, tikibase       | tikibase, rumdl       |
| Python     | Pyright, Ruff         | Ruff                  |
| Rust       | (your custom command) | (your custom command) |
| Shell      | ShellCheck            | shfmt                 |
| SQL        |                       | sqlfmt                |
| TOML       | Taplo                 | Taplo                 |
| TypeScript | Biome                 | Biome                 |
| YML        | Actionlint            | yamlfmt               |

<!-- SUPPORTED-STACKS-END -->

## Q & A

> Does Trident lock me into its tooling choices?

No. You can override which tools Trident runs in its config file.

> I want to add a stack or tool to Trident.

Send a pull request or open an issue!

## Installation

Trident ships as a single executable.
The Trident installer script downloads the Trident executable into the current
directory.
To install Trident into a particular directory,
run the installer from that directory.

### Linux and macOS

```sh
curl https://raw.githubusercontent.com/kevgo/trident/main/download.sh | sh
```

To download a specific version and/or save under a specific filename:

```sh
curl https://raw.githubusercontent.com/kevgo/trident/main/download.sh | sh -S -- [--version <version>] [--name <filename>]
```

### Windows PowerShell

Try to install and run Tricorder from a Bash environment on Windows,
for example the Git Bash.
Tricorder is a cross-platform development tool,
so it tries to run the tools it runs consistently inside a Bash environment on
all platforms.

If you don't have a Bash environment available,
you can also install using Powershell:

```powershell
Invoke-Expression (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/kevgo/trident/main/download.ps1" -UseBasicParsing).Content
```

### Compile from source

```sh
cargo install --git https://github.com/kevgo/trident
```

## Configuration

You can configure Trident using a **trident.json** or **trident.jsonc** file.
Both file types use JSONC format, so comments are always allowed.
The linked JSON-Schema provides help for supported settings.

<!-- CONFIG-EXAMPLE-START -->

```jsonc
{
  // link to the JSON schema for this file,
  // for auto-complete in VSCode and compatible editors
  "$schema": "https://github.com/kevgo/trident/raw/refs/heads/main/docs/schema.json",

  // These files are invisible to Trident.
  "ignore-files": ["node_modules/", "**/*.min.css"],

  // These tools always run.
  "global-lints": [
    {
      "name": "a custom lint with a name",
      "command": "echo Running a custom lint command"
    },
    {
      "command": "echo Just a command without name works too"
    }
  ],
  "global-fixes": [
    {
      "command": "echo Fixes are defined the same way as lints"
    }
  ],

  // Define the functional tests.
  "tests": [
    {
      "name": "a functional test",
      "command": "echo running this functional test"
    },
    {
      "name": "another test",
      "command": "echo running another test"
    }
  ],

  // configure the supported software stacks
  "stacks": {
    "css": {
      "lint": {
        "add": [
          {
            "name": "This lint gets run in addition to the built-in CSS lints",
            "command": "echo additional CSS lint"
          }
        ],
        "replace": [
          {
            "name": "This lint replaces the built-in CSS lints",
            "command": "echo The one and only CSS lint"
          }
        ]
      },
      "fix": {
        "before": [
          {
            "name": "This fix gets run before the built-in CSS fixes",
            "command": "echo initial CSS fix"
          },
        ],
        "replace": [
          {
            "name": "This fix replaces the built-in CSS fixes",
            "command": "echo My own custom CSS fix"
          }
        ],
        "after": [
          {
            "name": "This fix gets run after the built-in CSS fixes",
            "command": "echo final CSS fix"
          },
        ],
      }
    },
    "cucumber": {},
    "dockerfile": {},
    "go": {},
    "java": {},
    "json": {},
    "jsonc": {},
    "markdown": {},
    "python": {},
    "rust": {},
    "sql": {},
    "toml": {},
    "typescript": {},
    "unknown": {},
    "yml": {}
  },

  // configure the built-in tools
  //
  // Only applications that can receive file paths as CLI arguments have an "ignore-files" key.
  "applications": {
    "actionlint": {},
    "biome": {
      // enable or disable the entire biome tool
      "enabled": true,
      // don't process this file with Biome
      // (but other tools will process it)
      "ignore-files": ["not-by-biome.css"],
      "operations": {
        "lint": {
          // enable or disable the Biome lint functionality
          "enabled": true,
          // or don't lint this CSS file
          "ignore-files": ["dont-lint.css"]
        },
        "fix": {
          // enable or disable the Biome fix functionality
          "enabled": true,
          // don't fix this CSS file
          "ignore-files": ["dont-fix.css"]
        },
        "fix-unsafe": {
          // enable or disable all Biome fix-unsafe functionality
          "enabled": true,
          // don't fix-unsafe this CSS file
          "ignore-files": ["dont-fix-unsafe.css"]
        }
      }
    },
    "checkstyle": {},
    "delete_empty_folders": {},
    "gherkin_lint": {},
    "ghokin": {},
    "git_diff_check": {},
    "gofumpt": {},
    "golangci_lint": {},
    "hadolint": {},
    "keep_sorted": {},
    "pyright": {},
    "ruff": {},
    "rumdl": {},
    "shellcheck": {},
    "shfmt": {},
    "sqlfmt": {},
    "taplo": {},
    "tikibase": {},
  },

  // configure the Trident command behavior
  "commands": {
    "ci": {
      // Configure the tests that the "trident ci" command runs by default.
      // Tests are refered to by name, and need to be defined in the "tests" section above.
      "test": ["a functional test"]
    },
    "full": {},
    "pitstop": {},
    "test": {}
  }
}
```

<!-- CONFIG-EXAMPLE-END -->
