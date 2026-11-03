Feature: "trident precommit" treats staged deletions as changes

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
            "fix": {
              "after": [
                { "name": "md fix", "command": "echo running Markdown fix" }
              ]
            }
          },
          "rust": {
            "fix": {
              "replace": [
                { "name": "my fix", "command": "echo running Rust fix" }
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

  Scenario: only a staged deletion
    Given I ran "git rm main.rs"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      """
    And it prints the block
      """
      fix Rust (my fix)
      running Rust fix
      """
    And it prints to STDERR
      """
      0 Rust
      running 2 tasks
      """
    And it does not print
      """
      running Markdown fix
      """
    And the exit code is 0

  Scenario: staged rename into another stack
    Given I ran "git mv main.rs main.md"
    When executing "trident precommit --show=output"
    Then it prints the block
      """
      fix Rust (my fix)
      running Rust fix
      """
    And it prints the block
      """
      fix Markdown (md fix)
      running Markdown fix
      """
    And the exit code is 0

  @unix
  Scenario: unstaged deletion does not count
    Given I ran "rm main.rs"
    When executing "trident precommit --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
