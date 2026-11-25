Feature: "trident precommit" always exits 0

  Scenario: invalid config file
    Given a file "trident.json" with unix newlines
      """
      {
        "unknown-key": true
      }
      """
    When executing "trident precommit"
    Then it prints the block
      """
      config file (trident.json): unknown field `unknown-key`
      """
    And the exit code is 0
    And it runs no Git commands

  Scenario: not a Git repository
    When executing "trident precommit"
    Then it prints the block
      """
      not a git repository (no .git directory)
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
