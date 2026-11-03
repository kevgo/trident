Feature: "trident precommit" operates only on staged files that still exist in the workspace

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
    And a committed file "also-unrelated.md" with content
      """
      # Also unrelated
      """
    And a committed file "removed.md" with content
      """
      # Removed
      """

  Scenario: staged deletion
    Given I delete file "removed.md"
    And I ran "git add -A"
    When executing "trident precommit --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      """
    And it does not print
      """
      fix Markdown
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: staged deletion next to a staged change
    Given a file "added.md" with content
      """
      #     Added
      """
    And I delete file "removed.md"
    And I ran "git add -A"
    When executing "trident precommit --show=output"
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
    And file "added.md" now has content
      """
      # Added
      """

  Scenario: staged change to a file that was then deleted
    Given a file "added.md" with content
      """
      #     Added
      """
    And I change file "removed.md" to
      """
      # Changed
      """
    And I ran "git add -A"
    And I delete file "removed.md"
    When executing "trident precommit --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 0
    And file "added.md" now has content
      """
      # Added
      """
