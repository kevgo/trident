Feature: fix Rust

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

  Scenario: no custom fixes defined
    When executing "trident fix --show=output"
    Then it prints
      """
      delete empty folders
      """
    And it prints to STDERR
      """
      1 Rust, 1 other
      running 1 tools
      """
    And the exit code is 0
    And file "main.rs" is unchanged

  Scenario: rust-specific fixes
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
            }
          }
        }
      }
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      delete empty folders
      """
    And it prints the block
      """
      fix JSON (Biome)
      """
    And it prints the block
      """
      fix Rust (my custom fix)
      custom fix running
      """
    And it prints to STDERR
      """
      1 JSON, 1 Rust, 1 other
      running 3 tools
      """
    And the exit code is 0
    And file "main.rs" is unchanged
