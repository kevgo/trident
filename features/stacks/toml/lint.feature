Feature: lint TOML

  Background:
    Given a file "run-that-app" with unix newlines
      """
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """

  Scenario: valid TOML
    Given a file "main.toml" with unix newlines
      """
      key = "value"
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint TOML (Taplo)
      """
    And the exit code is 0
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted TOML
    Given a file "main.toml" with unix newlines
      """
      key =     "value"
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint TOML (Taplo)
      """
    And the exit code is 0
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid TOML
    Given a file "main.toml" with unix newlines
      """
      key = "
      """
    And a file "other.toml" with unix newlines
      """
      other = "
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint TOML (Taplo)
      error: invalid TOML
      """
    And the exit code is 1
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
