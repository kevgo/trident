Feature: precommit YML

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      yamlfmt 0.21.0
      """

  @this
  Scenario: valid YML
    Given a file "main.yml" with platform-specific newlines and content
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
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
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
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      git add -- main.yml other.yml
      """

  Scenario: invalid YML
    Given a file "main.yml" with unix newlines
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
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """
