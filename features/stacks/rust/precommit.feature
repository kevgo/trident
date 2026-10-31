Feature: precommit Rust

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a file "main.rs" with content
      """
      // some Rust code
      """
    And I ran "git add main.rs"

  Scenario: no custom fixes defined
    When executing "trident precommit --show=output"
    Then it prints
      """
      fix all (delete empty folders)
      """
    And it prints to STDERR
      """
      1 Rust
      running 1 tools
      """
    And the exit code is 0

  Scenario: a custom fix is defined
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
    When executing "trident precommit --show=output"
    Then it prints the block
      """
      fix Rust (my custom fix)
      custom fix running
      """
    And it prints to STDERR
      """
      1 Rust
      running 2 tools
      """
    And the exit code is 0
