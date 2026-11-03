Feature: "trident pitstop" fixes only existing changed files and lints all files of stacks with deleted or renamed files

  Background:
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """
    And a committed file "unrelated.md" with content
      """
      #     Unrelated
      """
    And a committed file "removed.md" with content
      """
      # Removed
      """

  Scenario: uncommitted deletion
    Given I delete file "removed.md"
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 3 tasks
      """
    And it prints the lines
      """
      lint Markdown (rumdl)
      """
    And it does not print
      """
      fix Markdown
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: uncommitted rename
    Given I change file "removed.md" to
      """
      #     Removed
      """
    And I ran "git add -A"
    And I ran "git commit -m change"
    And I rename file "removed.md" to "renamed.md"
    And I ran "git add -A"
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 4 tasks
      """
    And it prints the block
      """
      renamed.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And file "unrelated.md" is unchanged
    And file "removed.md" does not exist
    And file "renamed.md" now has content
      """
      # Removed
      """

  Scenario: deletion committed on the feature branch
    Given I ran "git checkout -b feature"
    And I delete file "removed.md"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 3 tasks
      """
    And it does not print
      """
      fix Markdown
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged
