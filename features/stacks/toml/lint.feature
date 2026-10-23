Feature: lint TOML

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """

  Scenario: valid TOML
    Given a file "main.toml" with content
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

  Scenario: unformatted TOML
    Given a file "main.toml" with content
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

  Scenario: invalid TOML
    Given a file "main.toml" with content
      """
      key = "
      """
    And a file "other.toml" with content
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
