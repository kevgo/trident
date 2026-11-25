Feature: lint Rust

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """
    And a file "main.rs" with unix newlines
      """
      // some Rust code
      """

  Scenario: no custom linters defined
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Rust, 1 other
      running 0 tasks
      """
    And it prints nothing to STDOUT
    And the exit code is 0
    And file "main.rs" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: rust-specific linters
    Given a file "trident.json" with unix newlines
      """
      {
        "stacks": {
          "rust": {
            "lint": {
              "replace": [
                {
                  "command": "echo 'custom linter running'",
                  "name": "my custom linter"
                }
              ]
            }
          }
        }
      }
      """
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 JSON, 1 Rust, 1 other
      running 2 tasks
      """
    And it prints the block
      """
      lint Rust (my custom linter)
      custom linter running
      """
    And the exit code is 0
    And file "main.rs" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
