@online
Feature: install Ghokin

  Scenario: not installed
    And a file "main.feature" with content
      """
      Feature:   foo

        Scenario:   bar
          Given a step
      """
    When executing "trident fix --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/antham/ghokin/releases/latest) ... ok
      """
    And it prints the lines
      """
      fix Cucumber (Ghokin)
      "main.feature" formatted
      """
    And the exit code is 0
    And file "main.feature" now has content
      """
      Feature: foo

        Scenario: bar
          Given a step
      """
    And file "run-that-app" now has an additional line matching
      """
      delete-empty-folders \d+\.\d+\.\d+
      ghokin \d+\.\d+\.\d+
      """
