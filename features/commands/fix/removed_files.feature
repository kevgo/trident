Feature: "trident fix" operates only on changed files that still exist in the workspace

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
    When executing "trident fix --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 1 tasks
      """
    And it prints
      """
      fix all (delete empty folders)
      """
    And it does not print
      """
      fix Markdown
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: uncommitted deletion next to a changed file
    Given a file "untracked.md" with content
      """
      #     Untracked
      """
    And I delete file "removed.md"
    When executing "trident fix --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      untracked.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      """
    And it does not print
      """
      removed.md
      """
    And it does not print
      """
      unrelated.md
      """
    And the exit code is 0
    And file "untracked.md" now has content
      """
      # Untracked
      """
    And file "unrelated.md" is unchanged

  Scenario: staged rename
    Given I change file "removed.md" to
      """
      #     Removed
      """
    And I ran "git add -A"
    And I ran "git commit -m change"
    And I rename file "removed.md" to "renamed.md"
    And I ran "git add -A"
    When executing "trident fix --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it does not print
      """
      unrelated.md
      """
    And the exit code is 0
    And file "unrelated.md" is unchanged
    And file "removed.md" does not exist
    And file "renamed.md" now has content
      """
      # Removed
      """

  Scenario: staged change to a file that was then deleted
    Given I change file "removed.md" to
      """
      # Changed
      """
    And I ran "git add -A"
    And I delete file "removed.md"
    When executing "trident fix --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 1 tasks
      """
    And it does not print
      """
      fix Markdown
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: deletion committed on the feature branch
    Given I ran "git checkout -b feature"
    And I delete file "removed.md"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident fix --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 1 tasks
      """
    And it does not print
      """
      fix Markdown
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: --scope=branch with a committed deletion next to a committed change
    Given I ran "git checkout -b feature"
    And a committed file "on-branch.md" with content
      """
      #     Branch
      """
    And I delete file "removed.md"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident fix --scope=branch --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it does not print
      """
      unrelated.md
      """
    And the exit code is 0
    And file "unrelated.md" is unchanged
    And file "on-branch.md" now has content
      """
      # Branch
      """
