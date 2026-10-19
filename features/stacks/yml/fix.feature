Feature: fix YML

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      prettier 3.7.0
      """

  Scenario: valid YML
    Given a file "main.yml" with content
      """
      key: value
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      fix YML (Prettier)
      """
    And the exit code is 0
    And file "main.yml" is unchanged

  Scenario: unformatted YML
    Given a file "main.yml" with content
      """
      key:     value
      """
    And a file "other.yml" with content
      """
      key:     other
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix YML (Prettier)
      """
    And the exit code is 0
    And file "main.yml" now has content
      """
      key: value
      """
    And file "other.yml" now has content
      """
      key: other
      """

  Scenario: invalid YML
    Given a file "main.yml" with content
      """
      key: "
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix YML (Prettier)
      [error] main.yml: SyntaxError: Missing closing "quote (1:6)
      """
    And the exit code is 2
    And file "main.yml" is unchanged
