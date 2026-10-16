Feature: precommit YML

  Background:
    Given a Git repository
    And a file "run-that-app" with content
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
    And I ran "git add main.yml"
    When executing "trident precommit --show=output"
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
    And I ran "git add main.yml other.yml"
    When executing "trident precommit --show=output"
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
    And I ran "git add main.yml"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix YML (Prettier)
      [error] main.yml: SyntaxError: Missing closing "quote (1:6)
      """
    And the exit code is 0
    And file "main.yml" is unchanged
