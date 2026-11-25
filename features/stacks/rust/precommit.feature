Feature: precommit Rust

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a file "main.rs" with unix newlines
      """
      // some Rust code
      """
    And I ran "git add main.rs"

  Scenario: no custom fixes defined
    When executing "trident precommit --show=output"
    Then it prints to STDERR
      """
      1 Rust
      running 1 tasks
      """
    And it prints
      """
      fix all (delete empty folders)
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """

  Scenario: a custom fix is defined
    Given a file "trident.json" with unix newlines
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
    Then it prints to STDERR
      """
      1 Rust
      running 2 tasks
      """
    And it prints the block
      """
      fix Rust (my custom fix)
      custom fix running
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """
