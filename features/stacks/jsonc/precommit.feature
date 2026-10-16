Feature: precommit JSONC

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      prettier 3.7.0
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "prettier": { "ignore-files": ["trident.json"] }
        }
      }
      """

  Scenario: formatted JSONC
    Given a file "main.jsonc" with content
      """
      { "key": "value" } // comment
      """
    And I ran "git add main.jsonc"
    When executing "trident precommit --show=output"
    Then it prints the block
      """
      fix JSONC (prettier)
      """
    And the exit code is 0
    And file "main.jsonc" is unchanged

  Scenario: unformatted JSONC
    Given a file "main.jsonc" with content
      """
      {  "key"  :  "value"  } // comment
      """
    And a file "other.jsonc" with content
      """
      {  "key"  :  "other"  } // comment
      """
    And I ran "git add main.jsonc other.jsonc"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix JSONC (prettier)
      """
    And the exit code is 0
    And file "main.jsonc" now has content
      """
      { "key": "value" } // comment
      """
    And file "other.jsonc" now has content
      """
      { "key": "other" } // comment
      """

  Scenario: invalid JSONC
    Given a file "main.jsonc" with content
      """
      { "key":
      """
    And I ran "git add main.jsonc"
    When executing "trident precommit --show=output"
    Then it prints the block
      """
      fix JSONC (prettier)
      Error formatting
      """
    And the exit code is 0
    And file "main.jsonc" is unchanged
