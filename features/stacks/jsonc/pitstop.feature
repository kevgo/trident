Feature: pitstop JSONC

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "prettier": { "ignore-files": ["trident.json"] }
        }
      }
      """

  Scenario: unformatted JSONC
    Given a file "main.jsonc" with content
      """
      {  "key"  :  "value"  } // comment
      """
    And a file "other.jsonc" with content
      """
      {  "key"  :  "other"  } // comment
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix JSONC (Prettier)
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
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      fix JSONC (Prettier)
      [error] main.jsonc: SyntaxError: Unexpected token (2:1)
      """
    And the exit code is 2
    And file "main.jsonc" is unchanged
