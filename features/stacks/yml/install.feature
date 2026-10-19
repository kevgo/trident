@online
Feature: install all YML tools

  Scenario: not installed
    Given a file "main.yml" with content
      """
      key:     value
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix YML (yamlfmt)
      """
    And the exit code is 0
    And file "main.yml" now has content
      """
      key: value
      """
    And file "run-that-app" now has an additional line matching
      """
      delete-empty-folders \d+\.\d+\.\d+
      yamlfmt \d+\.\d+\.\d+
      """
