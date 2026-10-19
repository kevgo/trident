Feature: precommit YML

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      yamlfmt 0.21.0
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
      fix YML (yamlfmt)
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
      fix YML (yamlfmt)
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

  @this
  Scenario: invalid YML
    Given a file "main.yml" with content
      """
      key: "
      """
    And I ran "git add main.yml"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix YML (yamlfmt)
      main.yml: yaml: line 2: found unexpected end of stream
      """
    And the exit code is 0
    And file "main.yml" is unchanged
