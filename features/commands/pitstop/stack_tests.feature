Feature: "trident pitstop" runs the tests of the stacks that have changes

  Background:
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a committed file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false },
          "rumdl": { "enabled": false }
        },
        "stacks": {
          "markdown": {
            "tests": [
              { "name": "text-runner", "command": "echo running Markdown tests" }
            ]
          },
          "rust": {
            "tests": [
              { "name": "backend", "command": "echo running Rust tests" }
            ]
          },
          "typescript": {
            "tests": [
              { "name": "frontend", "command": "echo running TypeScript tests" }
            ]
          }
        }
      }
      """
    And a committed file "main.rs" with content
      """
      // Rust code
      """
    And a committed file "main.ts" with content
      """
      // TypeScript code
      """
    And a committed file "README.md" with content
      """
      # Readme
      """

  Scenario: on main branch, no uncommitted files
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      test Markdown (text-runner)
      running Markdown tests
      """
    And it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it prints the block
      """
      test TypeScript (frontend)
      running TypeScript tests
      """
    And the exit code is 0

  Scenario: on main branch, has uncommitted files
    Given a file "uncommitted.rs" with content
      """
      // Rust code
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it does not print
      """
      running Markdown tests
      """
    And it does not print
      """
      running TypeScript tests
      """
    And the exit code is 0

  Scenario: on feature branch, branch has commits, no uncommitted files
    Given I ran "git checkout -b feature"
    And a committed file "branch.ts" with content
      """
      // TypeScript code
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      test TypeScript (frontend)
      running TypeScript tests
      """
    And it does not print
      """
      running Markdown tests
      """
    And it does not print
      """
      running Rust tests
      """
    And the exit code is 0

  Scenario: uncommitted deletion runs the global tools and the tests of the stack of the deleted file
    Given I ran "git rm main.rs"
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      lint all (git diff HEAD --check)
      """
    And it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it prints to STDERR
      """
      0 Rust
      running 3 tasks
      """
    And it does not print
      """
      running Markdown tests
      """
    And it does not print
      """
      running TypeScript tests
      """
    And the exit code is 0

  Scenario: branch only deletes a file
    Given I ran "git checkout -b feature"
    And I ran "git rm main.rs"
    And I ran "git commit --message=delete"
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      lint all (git diff HEAD --check)
      """
    And it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it prints to STDERR
      """
      0 Rust
      running 3 tasks
      """
    And it does not print
      """
      running Markdown tests
      """
    And the exit code is 0

  Scenario: deletion-only changes run the fixes, lints, and tests of that stack and all global tools
    Given I change file "trident.json" to
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "global-fixes": [
          { "name": "global fix", "command": "echo running global fix" }
        ],
        "global-lints": [
          { "name": "global lint", "command": "echo running global lint" }
        ],
        "tests": [
          { "name": "unit", "command": "echo running global tests" }
        ],
        "stacks": {
          "markdown": {
            "fix": {
              "after": [
                { "name": "md fix", "command": "echo running Markdown fix" }
              ]
            }
          },
          "rust": {
            "fix": {
              "after": [
                { "name": "my fix", "command": "echo running Rust fix" }
              ]
            },
            "lint": {
              "add": [
                { "name": "my lint", "command": "echo running Rust lint" }
              ]
            },
            "tests": [
              { "name": "backend", "command": "echo running Rust tests" }
            ]
          }
        },
        "commands": {
          "pitstop": {
            "test": ["unit"]
          }
        }
      }
      """
    And I ran "git add trident.json"
    And I ran "git commit --message=config"
    And I ran "git rm main.rs"
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      0 Rust
      running 8 tasks
      """
    And it prints the lines
      """
      fix all (delete empty folders)
      lint all (git diff HEAD --check)
      """
    And it prints the block
      """
      fix all (global fix)
      running global fix
      """
    And it prints the block
      """
      lint all (global lint)
      running global lint
      """
    And it prints the block
      """
      fix Rust (my fix)
      running Rust fix
      """
    And it prints the block
      """
      lint Rust (my lint)
      running Rust lint
      """
    And it prints the block
      """
      test all (unit)
      running global tests
      """
    And it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it does not print
      """
      running Markdown fix
      """
    And the exit code is 0

  Scenario: deletion together with another change runs fixes and lints
    Given I ran "git rm main.rs"
    And a file "uncommitted.ts" with content
      """
      // TypeScript code
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      """
    And it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it prints the block
      """
      test TypeScript (frontend)
      running TypeScript tests
      """
    And it does not print
      """
      running Markdown tests
      """
    And the exit code is 0

  Scenario: --scope=all runs fixes and lints despite deletion-only changes
    Given I ran "git rm main.rs"
    When executing "trident pitstop --scope=all --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      """
    And it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it prints the block
      """
      running Markdown tests
      """
    And the exit code is 0

  Scenario: --scope=uncommitted with deletion-only changes
    Given I ran "git rm main.rs"
    When executing "trident pitstop --scope=uncommitted --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      lint all (git diff HEAD --check)
      """
    And it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And the exit code is 0

  Scenario: branch deletes a file
    Given I ran "git checkout -b feature"
    And I ran "git rm main.rs"
    And I ran "git commit --message=delete"
    And a committed file "branch.ts" with content
      """
      // TypeScript code
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it prints the block
      """
      test TypeScript (frontend)
      running TypeScript tests
      """
    And it does not print
      """
      running Markdown tests
      """
    And the exit code is 0

  Scenario: --scope=all runs the tests of all stacks
    Given a file "uncommitted.rs" with content
      """
      // Rust code
      """
    When executing "trident pitstop --scope=all --show=output"
    Then it prints the block
      """
      running Markdown tests
      """
    And it prints the block
      """
      running Rust tests
      """
    And it prints the block
      """
      running TypeScript tests
      """
    And the exit code is 0

  Scenario: --test selects only the named global tests
    Given a file "uncommitted.rs" with content
      """
      // Rust code
      """
    And I change file "trident.json" to
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo running global tests" }
        ],
        "stacks": {
          "rust": {
            "tests": [
              { "name": "backend", "command": "echo running Rust tests" }
            ]
          }
        }
      }
      """
    When executing "trident pitstop --test=unit --show=output"
    Then it prints the block
      """
      test all (unit)
      running global tests
      """
    And it does not print
      """
      running Rust tests
      """
    And the exit code is 0

  Scenario: failing stack test
    Given a file "uncommitted.rs" with content
      """
      // Rust code
      """
    And I change file "trident.json" to
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "stacks": {
          "rust": {
            "tests": [
              { "name": "failing", "command": "echo stack test failed && exit 4" }
            ]
          }
        }
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      test Rust (failing)
      stack test failed
      """
    And the exit code is 4

  Scenario: stack tests do not run when a lint fails
    Given a file "uncommitted.rs" with content
      """
      // Rust code
      """
    And I change file "trident.json" to
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "stacks": {
          "rust": {
            "lint": {
              "add": [
                { "name": "failing", "command": "exit 3" }
              ]
            },
            "tests": [
              { "name": "backend", "command": "echo running Rust tests" }
            ]
          }
        }
      }
      """
    When executing "trident pitstop --show=output"
    Then it does not print
      """
      running Rust tests
      """
    And the exit code is 3

  @unix
  Scenario: stack tests run after the fixes of all stacks
    Given a file "uncommitted.rs" with content
      """
      // Rust code
      """
    And a file "uncommitted.toml" with content
      """
      key = "value"
      """
    And I change file "trident.json" to
      """
      {
        "applications": {
          "biome": { "enabled": false },
          "taplo": { "enabled": false }
        },
        "stacks": {
          "rust": {
            "tests": [
              { "name": "backend", "command": "cat marker.txt" }
            ]
          },
          "toml": {
            "fix": {
              "replace": [
                { "name": "slow fix", "command": "sleep 1 && echo written-by-fix > marker.txt" }
              ]
            }
          }
        }
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      test Rust (backend)
      written-by-fix
      """
    And the exit code is 0
