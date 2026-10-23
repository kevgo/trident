Feature: precommit TOML

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """

  Scenario: valid TOML
    Given a file "main.toml" with content
      """
      key = "value"
      """
    And I ran "git add main.toml"
    When executing "trident precommit --show=output"
    Then it prints the block
      """
      fix TOML (Taplo)
      """
    And the exit code is 0
    And file "main.toml" is unchanged

  Scenario: unformatted TOML
    Given a file "main.toml" with content
      """
      key =     "value"
      """
    And a file "other.toml" with content
      """
      key =     "other"
      """
    And I ran "git add main.toml other.toml"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix TOML (Taplo)
      """
    And the exit code is 0
    And file "main.toml" now has content
      """
      key = "value"
      """
    And file "other.toml" now has content
      """
      key = "other"
      """

  Scenario: invalid TOML
    Given a file "main.toml" with content
      """
      key = "
      """
    And I ran "git add main.toml"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix TOML (Taplo)
      error: invalid TOML
      """
    And the exit code is 0
    And file "main.toml" is unchanged
