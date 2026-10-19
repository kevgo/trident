@online
Feature: install all JSON tools

  Scenario: not installed
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      """
    And a file "main.json" with content
      """
      {"key":"value"}
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix JSON (Prettier)
      """
    And the exit code is 0
    And file "main.json" now has content
      """
      { "key": "value" }
      """
    And file "run-that-app" now has an additional line matching
      """
      prettier \d+\.\d+\.\d+
      """
