Feature: fix JSON

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      prettier 3.7.0
      """

  Scenario: valid JSON
    Given a file "main.json" with content
      """
      { "key": "value" }
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix JSON (Prettier)
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
      fix JSON (Prettier)
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
    Then it prints the block
      """
      fix JSON (Prettier)
      [error] main.json: SyntaxError: Unexpected token (2:1)
      """
    And the exit code is 2
    And file "main.json" is unchanged
