Feature: "trident postedit" lints all files of stacks with uncommitted deletions or renames

  Background:
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """
    And a committed file "unrelated.md" with content
      """
      missing header
      """
    And a committed file "also-unrelated.md" with content
      """
      # Also unrelated
      """
    And a committed file "removed.md" with content
      """
      # Removed
      """

  Scenario: uncommitted deletion
    Given I delete file "removed.md"
    When executing "trident postedit --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the lines
      """
      unrelated.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: uncommitted rename
    Given I rename file "removed.md" to "renamed.md"
    When executing "trident postedit --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it prints the lines
      """
      unrelated.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: deletion committed on the feature branch
    Given I ran "git checkout -b feature"
    And I delete file "removed.md"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident postedit --show=output"
    Then it prints to STDERR
      """
      running 1 tasks
      """
    And it does not print
      """
      unrelated.md
      """
    And the exit code is 0
    And all files are unchanged
