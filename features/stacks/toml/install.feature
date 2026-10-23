@online
Feature: install all TOML tools

  Scenario: not installed
    Given a file "main.toml" with content
      """
      key =      "value"
      """
    When executing "trident fix --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/tamasfe/taplo/releases/latest) ... ok
      """
    And it prints the lines
      """
      fix TOML (Taplo)
      """
    And the exit code is 0
    And file "main.toml" now has content
      """
      key = "value"
      """
    And file "run-that-app" now has an additional line matching
      """
      delete-empty-folders \d+\.\d+\.\d+
      taplo \d+\.\d+\.\d+
      """
