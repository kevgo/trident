Feature: "trident postedit" correctly handles deleted files

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
      #     Removed
      """

  Scenario: unstaged deletion
    Given I delete file "removed.md"
    When executing "trident postedit --show=verbose"
    Then it prints to STDERR
      """
      1 Markdown
      running 0 tasks
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: staged deletion
    Given I delete file "removed.md"
    And I ran "git add -A"
    When executing "trident lint --show=verbose"
    Then it prints to STDERR
      """
      1 Markdown
      running 0 tasks
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: unstaged deletion next to a changed file
    Given a file "unrelated.md" with content
      """
      #     Unrelated
      """
    And I delete file "removed.md"
    When executing "trident postedit --show=verbose"
    Then it prints to STDERR
      """
      2 Markdown
      running 1 tasks
      """
    And it prints the block matching
      """
      lint Markdown \(rumdl\)
      .*rumdl(.exe)? check unrelated.md$
      unrelated.md:1:2: \[MD019\] Multiple spaces \(5\) after # in heading \[\*\]
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: unstaged rename
    Given I rename file "removed.md" to "renamed.md"
    When executing "trident postedit --show=verbose"
    Then it prints to STDERR
      """
      2 Markdown
      running 1 tasks
      """
    And it prints the block matching
      """
      lint Markdown \(rumdl\)
      .*rumdl(.exe)? check renamed.md$
      renamed.md:1:2: \[MD019\] Multiple spaces \(5\) after # in heading \[\*\]
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: staged rename
    Given I rename file "removed.md" to "renamed.md"
    And I ran "git add -A"
    When executing "trident postedit --show=verbose"
    Then it prints to STDERR
      """
      2 Markdown
      running 1 tasks
      """
    And it prints the block matching
      """
      lint Markdown \(rumdl\)
      .*rumdl(.exe)? check renamed.md$
      renamed.md:1:2: \[MD019\] Multiple spaces \(5\) after # in heading \[\*\]
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: staged change to a file that was then deleted
    Given I change file "removed.md" to
      """
      # Changed
      """
    And I ran "git add -A"
    And I delete file "removed.md"
    When executing "trident postedit --show=verbose"
    Then it prints to STDERR
      """
      1 Markdown
      running 0 tasks
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: deletion committed on the feature branch
    Given I ran "git checkout -b feature"
    And I delete file "removed.md"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident postedit --show=verbose"
    Then it prints to STDERR
      """
      running 0 tasks
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: committed rename
    Given I ran "git checkout -b feature"
    And I rename file "removed.md" to "renamed.md"
    And I ran "git add -A"
    And I ran "git commit -m rename"
    When executing "trident postedit --show=verbose"
    Then it prints to STDERR
      """
      running 0 tasks
      """
    And the exit code is 0
    And all files are unchanged
