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
