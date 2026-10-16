# Trident

[![linux](https://github.com/kevgo/Trident/actions/workflows/linux.yml/badge.svg)](https://github.com/kevgo/trident/actions/workflows/linux.yml)
[![windows](https://github.com/kevgo/Trident/actions/workflows/windows.yml/badge.svg)](https://github.com/kevgo/trident/actions/workflows/windows.yml)

Trident runs all applicable type checkers, linters,
and formatters for the various file types in your codebase.
Concurrently for the fastest results.

Type checking, linting, and automated testing are essential
for high-quality software engineering, no matter who writes the code: AI,
humans, or code templates.

## Demo

If I run `trident lint --scope=all --show=output` on the Trident codebase,
using a 2021 MacBook Pro M1 running Linux,
it finishes in about 200ms and prints:

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

Trident has analyzed 315 files and executed 7 linters to verify
and improve them.
In this example:

- [Taplo](https://github.com/tamasfe/taplo) for the 3 TOML files
- [rumdl](https://github.com/rvben/rumdl) for the 4 Markdown files
- [gherkin-lint](https://github.com/gherkin-lint/gherkin-lint)
  for the 98 Cucumber files
- [actionlint](https://github.com/rhysd/actionlint)
  for the GitHub Action configuration
- [git diff --check](https://git-scm.com/docs/git-diff#Documentation/git-diff.txt---check)
  to detect unresolved merge conflict markers
- [Cargo clippy](https://doc.rust-lang.org/clippy)

You don't need to know about or even install these third-party linters.
Trident downloads them as needed.

## Why is it so fast

Trident is designed from the ground up for speed.

- It is written in Rust.
- It favors modern linters and formatters that execute quickly.
- It runs all tools concurrently.
- It passes each tool the exact files to process.
  This way each tool doesn't need to scan the entire file tree again to discover
  files it can process.

At the same time, Trident is extremely minimalistic:
It executes the tools directly on your machine, without Docker or emulation.

## Usage

Trident provides special commands
for all relevant phases of the software development workflow:

### `trident pitstop`

Run this command at any time during human development.
It first applies all safe automatic fixes to your current work,
then reports any remaining code problems that require (manual or AI) attention.

By default, this command reviews the smallest scope you currently work on:

- uncommitted changes exist: reviews only those
- no uncommitted changes but you are on a feature branch with changes:
  reviews all changes made on that branch
- otherwise: reviews the entire repo

To process a different set of files, use `--scope`.

You can also configure it to run certain tests during a pitstop:

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
The changes get formatted when being committed.

### `trident precommit`

This command runs inside the Git precommit hook and formats the staged changes
while they are getting committed.
It always exits with status code 0, so the commit always goes through,
even if there are formatting problems.

This command re-stages files that got formatted.
Re-staging operates on the entire file.
If you want to commit only part of a modified file,
run `trident pitstop` or `trident fix` to format everything
before partially staging your changes.
That way, Trident precommit won't introduce additional formatting changes
and won't need to re-stage the file.

### `trident ci`

This command runs inside your CI pipeline.
It ensures all code passes all checks, i.e. it fails if either:

- a linter reports an unresolved issue
- some code is unformatted, i.e. a formatter would modify a file
- a test defined in `trident.jsonc` fails

To run only some of those tests,
configure them in the config file or pass their names joined with `+`:

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

| Stack      | Linter                                                       |
| ---------- | ------------------------------------------------------------ |
| TypeScript | biome                                                        |
| CSS        | biome                                                        |
| Dockerfile | hadolint, dprint (when `dprint.json` is present)             |
| JSON       | prettier, dprint (when `dprint.json` is present)             |
| JSONC      | prettier, dprint (when `dprint.json` is present)             |
| YAML       | prettier, dprint (when `dprint.json` is present)             |
| Markdown   | rumdl                                                        |
| TOML       | taplo                                                        |
| Python     | ruff                                                         |
| Rust       | (please define your Rust commands as custom lints and fixes) |
| Go         | golangci-lint                                                |
| SQL        | sqlfmt                                                       |

## Q & A

> Does Trident lock me into its tooling choices?

No. You can override which tools Trident runs in its config file.

> I want to add a stack, linter, or formatter to Trident.

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

Tricorder is designed as a cross-platform development tool,
so it tries to run the tools it runs consistently inside a bash environment on
all platform.

If you don't have a Bash environment available:

```powershell
Invoke-Expression (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/kevgo/trident/main/download.ps1" -UseBasicParsing).Content
```

### Compile from source

```sh
cargo install --git https://github.com/kevgo/trident
```

## Configuration

You can configure Trident and define custom linters
and formatters in **trident.json** or **trident.jsonc**.
Both file types use JSONC format, so comments are allowed.
If both exist, **trident.json** takes precedence.

Use the linked JSON-Schema to get help on supported settings and their meaning.

<!-- DEFAULT-CONFIG-START -->

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
        "add": [
          {
            "name": "This fix gets run in addition to the built-in CSS fixes",
            "command": "echo additional CSS fix"
          },
        ],
        "replace": [
          {
            "name": "This fix replaces the built-in CSS fixes",
            "command": "echo The one and only CSS fix"
          }
        ]
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
      "enabled": true, // enable or disable the entire biome tool
      "operations": {
        "lint": {
          "enabled": true // enable or disable actionlint as a linter
        },
        "fix": {
          "enabled": true
        },
        "fix-unsafe": {
          "enabled": true
        }
      }
    },
    "checkstyle": {},
    "delete_empty_folders": {},
    "dprint": {},
    "gherkin_lint": {},
    "ghokin": {},
    "git_diff_check": {},
    "gofumpt": {},
    "golangci_lint": {},
    "hadolint": {},
    "keep_sorted": {},
    "prettier": {},
    "pyright": {},
    "ruff": {},
    "rumdl": {},
    "sqlfmt": {},
    "taplo": {},
    "text_runner": {},
    "tikibase": {},
  }
}
```

<!-- DEFAULT_CONFIG-END -->
