Feature: "trident full" runs stack-specific tests

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


  Scenario: runs the tests of all stacks even if only one stack has changes
    Given a file "uncommitted.rs" with content
      """
      // Rust code
      """
    When executing "trident full --show=output"
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

  Scenario: runs the tests of a stack whose last file the branch deleted
    Given I ran "git checkout -b feature"
    And I ran "git rm main.rs"
    And I ran "git commit --message=delete"
    When executing "trident full --show=output"
    Then it prints the block
      """
      test Rust (backend)
      running Rust tests
      """
    And the exit code is 0
