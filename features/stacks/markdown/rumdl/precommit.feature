Feature: precommit Markdown

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """

  Scenario: valid Markdown
    Given a file "main.md" with content
      """
      # Hello
      """
    And I ran "git add main.md"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """

  Scenario: unformatted JSON
    Given a file "main.md" with content
      """
      #     Hello
      """
    And I ran "git add main.md"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.md" now has content
      """
      # Hello
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      git add -- main.md
      """

  Scenario: unformatted Markdown with spaces in the filename
    Given a file "my file.md" with content
      """
      #     Hello
      """
    And I ran "git add -A"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "my file.md" now has content
      """
      # Hello
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      git add -- 'my file.md'
      """

  Scenario: invalid JSON
    Given a file "main.md" with content
      """
      # hello

      [e
      """
    And I ran "git add main.md"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """
