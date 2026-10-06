# Trident

_fuel additives for your AI workflow_

Type checking, linting, and automated testing are critical
for all software engineering, including AI engineering.
AI hallucinates non-existing APIs, dead code, and mismatching formatting styles.
Strict automated guardrails are the only things keeping your codebase from
turning into an untamable, hallucinated mess.

Trident solves this problem for you.
It runs all applicable type checkers, linters,
and formatters concurrently for the fastest results.

## Demo

Running `trident lint --scope=all --show=output` on the Trident codebase
finishes in about 500 ms and prints:

```sh
114 Cucumber, 3 JSON, 1 JSONC, 4 Markdown, 115 Rust, 2 TOML, 3 YML, 8 other
running 6 tools

lint Markdown (rumdl)
lint TOML (Taplo)
lint Git diff markers (git diff HEAD --check)
GitHub Actions (actionlint)
lint Cucumber (gherkin-lint)
cargo clippy
```

Trident has classified 203 files and executed 5 tools concurrently to lint them.
In this example:

- [Taplo](https://github.com/tamasfe/taplo) for the 3 TOML files
- [rumdl](https://github.com/rvben/rumdl) for the 4 Markdown files
- [gherkin-lint](https://github.com/gherkin-lint/gherkin-lint)
  for the 98 Cucumber files
- [actionlint](https://github.com/rhysd/actionlint)
  for the GitHub Action configuration
- [git diff --check](https://git-scm.com/docs/git-diff#Documentation/git-diff.txt---check)
  to detect unresolved merge conflict markers

These third-party linters don't need to be installed on the machine,
Trident downloads them if needed.

Trident is optimized for speed.
It favors modern linters and formatters that execute quickly,
runs them concurrently, and passes each tool the exact files to process,
so that the tools don't need to scan the codebase again to discover files to
process.

## Usage

Trident provides special commands
for specific phases of the software development workflow:

### `trident pitstop`

This command provides efficient support for interactive development.
It first applies all safe automatic fixes to your current work,
then reports any remaining code problems that require manual or AI attention.

By default, it reviews the smallest scope you currently work on:

- uncommitted changes exist: reviews only those
- no uncommitted changes but you are on a feature branch with changes:
  reviews all changes made on that branch
- otherwise: reviews the entire repo

To process a different set of files, use `--scope`.

To always run some tests as well, list them under `commands.pitstop.test`.

```jsonc
{
  "commands": {
    "pitstop": {
      "test": ["unit"]
    }
  }
}
```

### `trident full`

This command runs all available tools: all safe fixes, all lints, and all tests,
on all files.

To run only some tests, pass their names joined with `+`:

```sh
trident full --test=unit+cuke
```

To change the default tests, set `commands.full.test`.

### `trident postedit`

This command is the equivalent of `trident pitstop` for AI agents.
Each time your agent generates code, it checks it for problems.
This command does not format files because coding agents cache file contents
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

To run only some of those tests, pass their names joined with `+`:

```sh
trident ci --test=unit+cuke
```

To change the default tests, set `commands.ci.test`.

### `trident init:claude`

This command integrates Trident into coding agents harnesses
that follow Claude Code configuration, such as Claude Code, Codex, Code Puppy,
or Wibey.

Once configured, the agent runs `trident postedit` after it makes changes.
This command lints only the uncommitted files,
i.e. changes that the agent just made.

This helps the AI-generated generate cleaner code and fix possible bugs faster.

It works particularly well with custom AI-generated linters
that enforce invariants specific to your domain.

### `trident init:config`

Creates a scaffold of the Trident config file containing the default settings.

### `trident init:githook`

This command installs a
[Git pre-commit hook](https://git-scm.com/book/ms/v2/Customizing-Git-Git-Hooks)
that runs `trident precommit` before every commit.

### `trident update:tools`

This command updates the versions of third-party tools
that Trident uses to the latest available versions.

### `trident fix`

This command applies all safe automated fixes to your current work.
Fixes for different file types are processed concurrently,
multiple fixes for the same file type run sequentially.

### `trident fix-unsafe`

This command applies more aggressive automatic fixes to your current work.
These might change program behavior and should be verified.

### `trident lint`

This command runs all linters that apply to your current work.
All linters run in parallel.

### `trident test`

This command runs all tests defined in `trident.jsonc` in parallel.

To run only some tests, pass their names joined with `+`:

```sh
trident test --test=unit+cuke
```

To change the default tests, set `commands.test.test`.

### `--scope`

`ci`, `fix`, `fix-unsafe`, `lint`,
and `pitstop` accept `--scope` to choose which files they process:

- `uncommitted`: only uncommitted files (staged, unstaged, and untracked)
- `branch`: only files changed on the current branch
- `all`: all files in the current directory

`ci` defaults to `all`.
`fix`, `fix-unsafe`, `lint`,
and `pitstop` choose a default from the Git workspace:
uncommitted changes if present, otherwise files changed on the current branch,
otherwise all files.

## Supported stacks

| Stack      | Linter                                                                          |
| ---------- | ------------------------------------------------------------------------------- |
| TypeScript | biome                                                                           |
| CSS        | biome                                                                           |
| Dockerfile | hadolint, dprint (when `dprint.json` is present)                                |
| JSON       | prettier, dprint (when `dprint.json` is present)                                |
| JSONC      | prettier, dprint (when `dprint.json` is present)                                |
| YAML       | prettier, dprint (when `dprint.json` is present)                                |
| Markdown   | rumdl                                                                           |
| TOML       | taplo                                                                           |
| Python     | ruff                                                                            |
| Rust       | (none, please define your Rust commands as custom lints and fixes)              |
| Go         | golangci-lint                                                                   |
| Java       | checkstyle                                                                      |
| SQL        | sqlfmt                                                                          |

## Q & A

> Does Trident lock me into its tooling choices?

No. You can override which tools run in the Trident config file.

> I want to add a linter or formatter to Trident.

Send a pull request or open an issue!

## Installation

The installer script downloads the Trident executable into the current
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
Comments and trailing commas are allowed in either file.
If both exist, **trident.json** takes precedence.

<!-- DEFAULT-CONFIG-START -->

```jsonc
{
  // link to the JSON schema for this file,
  // for auto-complete in VSCode and compatible editors
  "$schema": "https://github.com/kevgo/trident/raw/refs/heads/main/docs/schema.json",

  // These files are invisible to Trident.
  "ignore-files": ["node_modules/", "**/*.min.css"],

  // These tools always run.
  "global-lints": [],
  "global-fixes": [],

  // Define the functional tests.
  "tests": [],

  // configure the supported software stacks
  "stacks": {},

  // configure the built-in tools
  //
  // Only applications that can receive file paths as CLI arguments have an "ignore-files" key.
  "applications": {}
}
```

<!-- DEFAULT_CONFIG-END -->
