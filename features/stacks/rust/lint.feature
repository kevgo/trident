Feature: lint Rust

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """
    And a file "main.rs" with content
      """
      // some Rust code
      """

  Scenario: no custom linters defined
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And it prints to STDERR
      """
      1 Rust, 1 other
      running 0 tools
      """
    And the exit code is 0
    And file "main.rs" is unchanged

  Scenario: rust-specific linters
    Given a file "trident.json" with content
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
    Then it prints the block
      """
      lint Rust (my custom linter)
      custom linter running
      """
    And it prints to STDERR
      """
      1 JSON, 1 Rust, 1 other
      running 2 tools
      """
    And the exit code is 0
    And file "main.rs" is unchanged
