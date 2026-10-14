# Trident changelog

## 0.1.0

New name: renamed the app from Tricorder to Trident to avoid a protected trademark.

New environments:
  - Windows support
  - Git Town support: Trident know knows about Git Town branch hierarchy

New config file:
  - the config file is in JSONC format now because there is too much nesting for TOML
  - many keys in the config file are simplified
  - the `stack` field on `custom-lints` and `custom-fixes` is removed; use `stack.<name>` with `lint`/`add-lint` and `fix`/`add-fix` to replace or extend a stack's built-in tools
  - the `keep-sorted` config key now lives under `applications`
  - unknown keys in the config file are now an error
  - the `ignore` config key is renamed to `ignore-files`
  - JSON-Schema for the config file is available at https://github.com/kevgo/trident/raw/refs/heads/main/docs/schema.json.
  - all tool definitions in the config file can now have an optional name
  - you can now enable/disable and ignore files for individual tools or operations of each tool
  - `commands.<name>.test` sets the tests that command runs when you omit `--test`, e.g. `"commands": { "pitstop": { "test": ["unit"] } }`

New commands:
  - `trident postedit`: a new command optimized for running after code was changed, for example insided the agentic loop. Lints only the uncommitted files.
  - `trident update:tools` updates third-party tool versions to the latest available
  - `trident full` runs all lints, fixes, and tests on all files
  - `--sequential` flag makes Tricorder run all operations sequentially
  - splits the `init` command into `init:config`, `init:claude`, and `init:githook`
  - new `test` command runs the tasks defined in the config `tests` key in parallel

New stacks:
  - `rust` but you need to define the commands for linting and fixing yourself because there is too much customization needed.
  - GitHub Actions config files via actionlint
  - Dockerfiles via hadolint
  - support for `dprint` if file `dprint.json` is present

New behaviors:
  - auto-scope: `ci`, `fix`, `fix-unsafe`, `lint`, and `pitstop` now process files based on the current situation, and accept `--scope=uncommitted|branch|all` to override the file selection
  - the `--test` flag allows running test scripts in addition to linters and fixes
  - now versions `delete-empty-folders` through the `run-that-app` file
  - `fix-unsafe` now prints tools as `unsafe-fix <stack> (<name>)`


#### Bug fixes

- `precommit` now always exits with code 0, even if the Trident config file is invalid
- custom stack tools now print `{operation} {stack} ({name})`, matching built-in tools

## 0.0.17 (2026-08-15)

- new `--show=verbose` option, prints the full command line (program and arguments) of every command Trident runs
- custom fixes with a `stack` now run only if the current scope contains at least one file of that stack

## 0.0.16 (2026-08-10)

- app: [keep-sorted](https://github.com/google/keep-sorted), since this requires grepping for markers, it needs to be enabled via the new `[keep-sorted]` config option
- app: `git diff --check` if the current codebase is in a Git repository

## 0.0.15 (2026-08-04)

- bug fixes

## 0.0.14 (2026-08-04)

- The "precommit" command now stages files it modifies.
  This allows running "trident precommit" directly as the Git precommit hook.
- The "ghokin" app now gets called with the files to format.
  Requires at least Ghokin v3.10.0.
- The new `exclude` key in the Trident config file ("trident.toml") allows ignoring files.

## 0.0.13 (2026-07-15)

- the "precommit" command fixes only staged files
- calls out non-UTF8 file paths
- runs the real Prettier package via NodeJS

## 0.0.12 (2026-07-13)

- provide carrier version in `run-that-app`
- the CI command now uses the default visibility `names`
- app: Text-Runner
- JSONC stack

## 0.0.11 (2026-07-05)

- new RTA backend

## 0.0.10 (2026-06-26)

- `trident ci` command
- app: Tikibase

## 0.0.9 (2026-06-25)

- `trident fix-unsafe` command

## 0.0.8 (2026-06-24)

- `trident pitstop` command

## 0.0.7 (2026-06-23)

- rename `trident check` to `trident lint` because it only runs linters
  and doesn't verify formatting anymore
- rename `[[custom-linters]]` in the config file to `[[custom-lints]]`
- rename `[[custom-fixers]]` in the config file to `[[custom-fixes]]`

## 0.0.6 (2026-06-22)

- custom fixes

## 0.0.5 (2026-06-19)

- simplified config file format
- executes delete-empty-folders separately before the other fixers
  because it isn't stack specific

## 0.0.4 (2026-06-18)

- the new `trident precommit` command never fails even if the code is invalid
- now runs `pyright` for Python stacks,
  but only if file `pyrightconfig.json` exists
- updates `delete-empty-folders` to v0.0.2

## 0.0.3 (2026-06-17)

- Support for custom linters in `trident.toml`.
- Removed checking for formatting from `trident lint`.
  Given that not all stacks support format checking,
  and there are custom formatters,
  the only way to guarantee correct formatting is to run `trident fix`.
- Renamed `trident format` to `trident fix` because it formats
  and also fixes code smells.
- `trident postgenerate` for running inside the agentic loop.
- `trident precommit` for running inside Git's precommit hook.

## 0.0.2 (2026-06-16)

- Ruff runs both the formatter and linter
- always runs the `delete-empty-folder` fixer

## 0.0.1 (2026-06-15)

- commands: check, format, init
- stacks: CSS, Cucumber, Go, Java, JSON, Markdown, Python, SQL, TOML, TypeScript
- tools: Biome, CheckStyle, Gherkin-Lint, Ghokin, Gofumpt, Golangci-Lint,
  Prettier, Ruff, Rumdl, Sqlfmt, Taplo
