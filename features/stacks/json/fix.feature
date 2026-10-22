Feature: fix JSON

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: valid JSON
    Given a file "main.json" with content
      """
      { "key": "value" }
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix JSON (Biome)
      """
    And the exit code is 0
    And file "main.json" is unchanged

  Scenario: unformatted JSON
    Given a file "main.json" with content
      """
      {"key":"value"}
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix JSON (Biome)
      """
    And the exit code is 0
    And file "main.json" now has content
      """
      { "key": "value" }
      """

  Scenario: invalid JSON
    Given a file "main.json" with content
      """
      { "key":
      """
    When executing "trident fix --show=output"
    Then it prints the lines matching
      """
      fix JSON \(Biome\)
      Expected an array, an object, or a literal but instead found the end of the file
      """
    And the exit code is 1
    And file "main.json" is unchanged
