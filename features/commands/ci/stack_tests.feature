Feature: "trident ci" runs stack-specific tests

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
    And I ran "git checkout -b feature"
    And a committed file "branch.ts" with content
      """
      // TypeScript code
      """

  Scenario: runs the tests of all stacks by default
    When executing "trident ci --show=output"
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

  Scenario: --scope=branch runs only the tests of the stacks changed on the branch
    When executing "trident ci --scope=branch --show=output"
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

  Scenario: failing stack test
    Given I change file "trident.json" to
      """
      {
        "applications": {
          "biome": { "enabled": false },
          "rumdl": { "enabled": false }
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
    When executing "trident ci --show=output"
    Then it prints the block
      """
      test Rust (failing)
      stack test failed
      """
    And the exit code is 4
