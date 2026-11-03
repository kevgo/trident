Feature: stack-specific tests

  Scenario: not a Git repository runs the tests of all existing stacks
    Given a file "trident.json" with content
      """
      {
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
    And a file "main.rs" with content
      """
      // Rust code
      """
    And a file "README.md" with content
      """
      # Readme
      """
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 2 tasks
      """
    And it prints the block
      """
      test Markdown (text-runner)
      running Markdown tests
      """
    And it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it does not print
      """
      running TypeScript tests
      """
    And the exit code is 0

  Scenario: global tests run together with stack tests
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running global tests" }
        ],
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
    And a file "main.rs" with content
      """
      // Rust code
      """
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 2 tasks
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
    And the exit code is 0

  Scenario: multiple tests for a stack
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "rust": {
            "tests": [
              { "name": "one", "command": "echo first Rust test" },
              { "command": "echo unnamed Rust test" }
            ]
          }
        }
      }
      """
    And a file "main.rs" with content
      """
      // Rust code
      """
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 2 tasks
      """
    And it prints the block
      """
      test Rust (one)
      first Rust test
      """
    And it prints the block
      """
      test Rust (echo unnamed Rust test)
      unnamed Rust test
      """
    And the exit code is 0

  Scenario: failing stack test
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "rust": {
            "tests": [
              { "name": "failing", "command": "echo stack test failed && exit 4" }
            ]
          }
        }
      }
      """
    And a file "main.rs" with content
      """
      // Rust code
      """
    When executing "trident test --show=output"
    Then it prints the block
      """
      test Rust (failing)
      stack test failed
      """
    And the exit code is 4

  Scenario: --test selects only the named global tests
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running global tests" }
        ],
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
    And a file "main.rs" with content
      """
      // Rust code
      """
    When executing "trident test --test=unit --show=output"
    Then it prints to STDERR
      """
      running 1 tasks
      """
    And it prints the block
      """
      test all (unit)
      running global tests
      """
    And it does not print
      """
      running Rust tests
      """
    And the exit code is 0

  Scenario: --test does not know stack tests
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running global tests" }
        ],
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
    And a file "main.rs" with content
      """
      // Rust code
      """
    When executing "trident test --test=backend"
    Then it prints
      """
      unknown test: backend
      available tests: unit
      """
    And the exit code is 1

  Scenario: commands.test.test selects global tests and keeps the stack tests
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running global unit tests" },
          { "name": "cuke", "command": "echo running global cuke tests" }
        ],
        "stacks": {
          "rust": {
            "tests": [
              { "name": "backend", "command": "echo running Rust tests" }
            ]
          }
        },
        "commands": {
          "test": {
            "test": ["unit"]
          }
        }
      }
      """
    And a file "main.rs" with content
      """
      // Rust code
      """
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 2 tasks
      """
    And it prints the block
      """
      test all (unit)
      running global unit tests
      """
    And it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And it does not print
      """
      running global cuke tests
      """
    And the exit code is 0

  Scenario: ignored files do not trigger stack tests
    Given a file "trident.json" with content
      """
      {
        "ignore-files": ["vendor/"],
        "stacks": {
          "rust": {
            "tests": [
              { "name": "backend", "command": "echo running Rust tests" }
            ]
          }
        }
      }
      """
    And a file "vendor/main.rs" with content
      """
      // Rust code
      """
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 0 tasks
      """
    And it does not print
      """
      running Rust tests
      """
    And the exit code is 0

  Scenario: deleting an ignored file does not trigger stack tests
    Given a Git repository
    And a committed file "trident.json" with content
      """
      {
        "ignore-files": ["vendor/"],
        "stacks": {
          "rust": {
            "tests": [
              { "name": "backend", "command": "echo running Rust tests" }
            ]
          }
        }
      }
      """
    And a committed file "vendor/main.rs" with content
      """
      // Rust code
      """
    And I ran "git rm vendor/main.rs"
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 0 tasks
      """
    And it does not print
      """
      running Rust tests
      """
    And the exit code is 0

  Scenario: deleting an ignored file runs the global tools like any other ignored change
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a committed file "trident.json" with content
      """
      {
        "ignore-files": ["vendor/"]
      }
      """
    And a committed file "vendor/main.rs" with content
      """
      // Rust code
      """
    And I ran "git rm vendor/main.rs"
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      """
    And the exit code is 0

  Scenario Outline: commands that only fix or only lint do not run stack tests
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a committed file "trident.json" with content
      """
      {
        "stacks": {
          "rust": {
            "tests": [
              { "name": "backend", "command": "echo running Rust tests" }
            ]
          }
        }
      }
      """
    And a file "main.rs" with content
      """
      // Rust code
      """
    When executing "trident <COMMAND> --show=output"
    Then it does not print
      """
      running Rust tests
      """
    And the exit code is 0

    Examples:
      | COMMAND    |
      | fix        |
      | fix-unsafe |
      | lint       |
      | postedit   |

  Scenario: precommit does not run stack tests
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a committed file "trident.json" with content
      """
      {
        "stacks": {
          "rust": {
            "tests": [
              { "name": "backend", "command": "echo running Rust tests" }
            ]
          }
        }
      }
      """
    And a file "main.rs" with content
      """
      // Rust code
      """
    And I ran "git add main.rs"
    When executing "trident precommit --show=output"
    Then it does not print
      """
      running Rust tests
      """
    And the exit code is 0

  Scenario Outline: deleting a file runs the custom fixes and lints of its stack
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a committed file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "stacks": {
          "rust": {
            "fix": {
              "replace": [
                { "name": "my fix", "command": "echo running Rust fix" }
              ]
            },
            "lint": {
              "replace": [
                { "name": "my lint", "command": "echo running Rust lint" }
              ]
            }
          }
        }
      }
      """
    And a committed file "main.rs" with content
      """
      // Rust code
      """
    And a committed file "other.rs" with content
      """
      // Rust code
      """
    And I ran "git rm main.rs"
    When executing "trident <COMMAND> --show=output"
    Then it prints the lines
      """
      <OUTPUT>
      """
    And the exit code is 0

    Examples:
      | COMMAND  | OUTPUT            |
      | fix      | running Rust fix  |
      | lint     | running Rust lint |
      | pitstop  | running Rust fix  |
      | pitstop  | running Rust lint |
      | postedit | running Rust lint |
