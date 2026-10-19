Feature: precommit JSON

  Background:
    Given a Git repository
    And a file "run-that-app" with content
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
    And I ran "git add main.json"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.json" is unchanged

  Scenario: unformatted JSON
    Given a file "main.json" with content
      """
      {"key":"value"}
      """
    And I ran "git add main.json"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
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
    And I ran "git add main.json"
    When executing "trident precommit"
    Then it prints the block
      """
      [error] main.json: SyntaxError: Unexpected token (2:1)
      """
    And the exit code is 0
    And file "main.json" is unchanged
