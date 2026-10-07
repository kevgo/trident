@online
Feature: install Gherkin Lint

  Scenario: not installed
    Given a file "features/one.feature" with content
      """
      Feature: one

        Scenario: one
          Given a step
      """
    And a file ".gherkin-lintrc" with content
      """
      {}
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Cucumber (gherkin-lint)
      """
    And the exit code is 0
    And file "run-that-app" now has an additional line matching
      """
      gherkin-lint \d+\.\d+\.\d+
      node \d+\.\d+\.\d+
      """
