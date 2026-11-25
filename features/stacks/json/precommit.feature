Feature: precommit JSON

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: valid JSON
    Given a file "main.json" with unix newlines
      """
      { "key": "value" }
      """
    And I ran "git add main.json"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.json" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """

  Scenario: unformatted JSON
    Given a file "main.json" with unix newlines
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
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      git add -- main.json
      """

  Scenario: invalid JSON
    Given a file "main.json" with unix newlines
      """
      { "key":
      """
    And I ran "git add main.json"
    When executing "trident precommit"
    Then it prints the lines matching
      """
      Expected an array, an object, or a literal but instead found the end of the file.
      """
    And the exit code is 0
    And file "main.json" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """
