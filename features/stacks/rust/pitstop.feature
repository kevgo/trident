Feature: pitstop Rust

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

  Scenario: no custom tools defined
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 Rust, 1 other
      running 1 tasks
      """
    And it prints
      """
      fix all (delete empty folders)
      """
    And the exit code is 0
    And file "main.rs" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: custom linters and fixes defined
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "rust": {
            "fix": {
              "replace": [
                {
                  "command": "echo 'custom fix running'",
                  "name": "my custom fix"
                }
              ]
            },
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
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 JSON, 1 Rust, 1 other
      running 5 tasks
      """
    And it prints the block
      """
      fix Rust (my custom fix)
      custom fix running
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
