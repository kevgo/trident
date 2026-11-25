Feature: fix YML

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      yamlfmt 0.21.0
      """

  Scenario: valid YML
    Given a file "main.yml" with content
      """
      key: value
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      fix YML (yamlfmt)
      """
    And the exit code is 0
    And file "main.yml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted YML
    Given a file "main.yml" with unix newlines
      """
      key:     value
      """
    And a file "other.yml" with unix newlines
      """
      key:     other
      """
    When executing "trident fix --show=output"
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
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid YML
    Given a file "main.yml" with unix newlines
      """
      key: "
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix YML (yamlfmt)
      main.yml: yaml: line 2: found unexpected end of stream
      """
    And the exit code is 1
    And file "main.yml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
