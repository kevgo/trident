Feature: fix TOML

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """

  Scenario: unformatted TOML
    Given a file "main.toml" with content
      """
      key =     "value"
      """
    And a file "other.toml" with content
      """
      key =     "other"
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix TOML (Taplo)
      lint TOML (Taplo)
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

  Scenario: unformatted TOML with lint errors
    Given a file "main.toml" with content
      """
      key = 1
      key = 2
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix TOML (Taplo)
      lint TOML (Taplo)
      error: conflicting keys
      """
    And the exit code is 1
    And file "main.toml" is unchanged
