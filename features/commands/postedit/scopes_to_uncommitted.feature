Feature: "trident postedit" lints only uncommitted files

  Background:
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """

  @this
  Scenario: no uncommitted files
    Given a committed file "committed.md" with content
      """
      missing header
      """
    When executing "trident postedit --show=output"
    Then it prints to STDERR
      """
      running 1 tools
      """
    And it prints the lines
      """
      lint Git diff markers (git diff HEAD --check)
      """
    And it does not print
      """
      lint Markdown
      """
    And the exit code is 0

  Scenario: has uncommitted files
    Given a committed file "already_committed.md" with content
      """
      missing header
      """
    And a committed file "modified.md" with content
      """
      # correct header
      """
    And I change file "modified.md" to
      """
      missing header
      """
    And a file "untracked.md" with content
      """
      missing header
      """
    When executing "trident postedit --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tools
      """
    And it prints the lines
      """
      lint Markdown (rumdl)
      """
    And it prints the lines
      """
      modified.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And it prints the lines
      """
      untracked.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And it does not print
      """
      already_committed.md
      """
    And the exit code is 1

  Scenario: untracked file with spaces in the name
    Given a file "my file.md" with content
      """
      missing header
      """
    When executing "trident postedit --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tools
      """
    And it prints the lines
      """
      my file.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And the exit code is 1
