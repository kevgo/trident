Feature: "trident test" runs the tests of the stacks that have changes

  Background:
    Given a Git repository
    And a committed file "trident.json" with content
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
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 3 tasks
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
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 1 tasks
      """
    And it prints the block
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

  Scenario: uncommitted files of multiple stacks
    Given a file "uncommitted.md" with content
      """
      # Uncommitted
      """
    And a file "uncommitted.rs" with content
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

  Scenario: uncommitted files of a stack without tests
    Given a file "uncommitted.py" with content
      """
      # Python code
      """
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 0 tasks
      """
    And it prints nothing to STDOUT
    And the exit code is 0

  Scenario: on feature branch, branch has no commits, no uncommitted files
    Given I ran "git checkout -b feature"
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 3 tasks
      """
    And the exit code is 0

  Scenario: on feature branch, branch has commits, no uncommitted files
    Given I ran "git checkout -b feature"
    And a committed file "branch.ts" with content
      """
      // TypeScript code
      """
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 1 tasks
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
    And it does not print
      """
      running Rust tests
      """
    And the exit code is 0

  Scenario: on feature branch, branch has commits, has uncommitted files
    Given I ran "git checkout -b feature"
    And a committed file "branch.ts" with content
      """
      // TypeScript code
      """
    And a file "uncommitted.md" with content
      """
      # Uncommitted
      """
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 1 tasks
      """
    And it prints the block
      """
      test Markdown (text-runner)
      running Markdown tests
      """
    And it does not print
      """
      running Rust tests
      """
    And it does not print
      """
      running TypeScript tests
      """
    And the exit code is 0

  Scenario: --scope=all overrides the inferred scope
    Given a file "uncommitted.rs" with content
      """
      // Rust code
      """
    When executing "trident test --scope=all --show=output"
    Then it prints to STDERR
      """
      running 3 tasks
      """
    And the exit code is 0

  Scenario: --scope=branch includes committed and uncommitted changes
    Given I ran "git checkout -b feature"
    And a committed file "branch.ts" with content
      """
      // TypeScript code
      """
    And a file "uncommitted.md" with content
      """
      # Uncommitted
      """
    When executing "trident test --scope=branch --show=output"
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
      test TypeScript (frontend)
      running TypeScript tests
      """
    And it does not print
      """
      running Rust tests
      """
    And the exit code is 0

  Scenario: --scope=uncommitted without uncommitted changes
    When executing "trident test --scope=uncommitted --show=output"
    Then it prints to STDERR
      """
      running 0 tasks
      """
    And it prints nothing to STDOUT
    And the exit code is 0
