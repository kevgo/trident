Feature: unsafe-fix TOML

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """
    And a file "main.toml" with content
      """
      [package
      name="demo"
      version   = "0.1.0"
      """

  Scenario: fix-unsafe
    When executing "trident fix-unsafe --show=output"
    Then it prints the lines
      """
      unsafe-fix TOML (Taplo)
      """
    And the exit code is 0
    And file "main.toml" now has content
      """
      [package
      name="demo"
      version = "0.1.0"
      """

  Scenario: fix
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix TOML (Taplo)
      error: invalid TOML
      """
    And the exit code is 1
    And file "main.toml" is unchanged
