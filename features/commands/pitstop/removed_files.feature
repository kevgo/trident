Feature: "trident pitstop" correctly handles deleted files

  Background:
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      rumdl 0.2.14
      """
    And a committed file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false },
          "delete-empty-folders": { "enabled": false },
          "git-diff-check": { "enabled": false }
        }
      }
      """
    And a committed file "removed.md" with content
      """
      #    Removed
      """

  Scenario: uncommitted deletion
    Given I delete file "removed.md"
    When executing "trident pitstop --show=verbose"
    Then it prints to STDERR
      """
      1 Markdown
      running 0 tasks
      """
    And the exit code is 0
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
    When executing "trident pitstop --show=verbose"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it prints the block matching
      """
      fix Markdown \(rumdl\)
      .*rumdl fmt renamed.md$
      renamed.md:1:2: \[MD019\] Multiple spaces \(5\) after # in heading \[fixed\]
      """
    And it prints the block matching
      """
      lint Markdown \(rumdl\)
      .*rumdl check renamed.md$
      """
    And the exit code is 0
    And file "renamed.md" now has content
      """
      # Removed
      """
    And file "removed.md" does not exist

  Scenario: committed deletion
    Given I ran "git checkout -b feature"
    And I delete file "removed.md"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident pitstop --show=verbose"
    Then it prints to STDERR
      """
      1 Markdown
      running 0 tasks
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: uncommitted deletion next to a changed file
    Given a file "untracked.md" with content
      """
      #     Untracked
      """
    And I delete file "removed.md"
    When executing "trident pitstop --show=verbose"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it prints the block matching
      """
      fix Markdown \(rumdl\)
      .*rumdl fmt untracked.md$
      untracked.md:1:2: \[MD019\] Multiple spaces \(5\) after # in heading \[fixed\]
      """
    And it prints the block matching
      """
      lint Markdown \(rumdl\)
      .*rumdl check untracked.md$
      """
    And the exit code is 0
    And file "untracked.md" now has content
      """
      # Untracked
      """
