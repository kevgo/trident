Feature: pitstop JSON

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      prettier 3.7.0
      """

  Scenario: unformatted JSON
    Given a file "main.json" with content
      """
      {"key":"value"}
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix JSON (Prettier)
      """
    And the exit code is 0
    And file "main.json" now has content
      """
      { "key": "value" }
      """

  Scenario: unformatted JSON with lint errors
    Given a file "main.json" with content
      """
      {
      "key":"value",
      "key":"value"
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      fix JSON (Prettier)
      """
    And the exit code is 0
    And file "main.json" now has content
      """
      {
        "key": "value",
        "key": "value"
      }
      """
