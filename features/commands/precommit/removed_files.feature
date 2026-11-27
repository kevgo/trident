Feature: "trident precommit" correctly handles deleted files

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
        },
        "ignore-files": ["ignored.md"]
      }
      """
    And a committed file "removed.md" with content
      """
      #     Removed
      """

  Scenario: unstaged deletion
    Given I delete file "removed.md"
    When executing "trident precommit --show=verbose"
    Then it prints nothing to STDERR
    And the exit code is 0
    And all files are unchanged

  Scenario: staged deletion
    Given I delete file "removed.md"
    And I ran "git add -A"
    When executing "trident precommit --show=verbose"
    Then it prints to STDERR
      """
      1 Markdown
      """
    And it prints nothing to STDOUT
    And the exit code is 0
    And all files are unchanged

  Scenario: staged deletion next to a staged change
    Given a file "added.md" with content
      """
      #     Added
      """
    And I delete file "removed.md"
    And I ran "git add -A"
    When executing "trident precommit --show=verbose"
    Then it prints the lines to STDERR
      """
      2 Markdown
      running 1 tasks
      """
    And it prints the block matching
      """
      fix Markdown \(rumdl\)
      .*rumdl(.exe)? fmt added.md$
      added.md:1:2: \[MD019\] Multiple spaces \(5\) after # in heading \[fixed\]
      """
    And the exit code is 0
    And file "added.md" now has content
      """
      # Added
      """

  Scenario: unstaged rename
    Given I rename file "removed.md" to "renamed.md"
    When executing "trident precommit --show=verbose"
    Then it prints nothing to STDERR
    And it prints nothing to STDOUT
    And the exit code is 0
    And all files are unchanged

  Scenario: staged rename
    Given I rename file "removed.md" to "renamed.md"
    And I ran "git add -A"
    When executing "trident precommit --show=verbose"
    Then it prints to STDERR
      """
      2 Markdown
      running 1 tasks
      """
    And it prints the block matching
      """
      fix Markdown \(rumdl\)
      .*rumdl(.exe)? fmt renamed.md$
      renamed.md:1:2: \[MD019\] Multiple spaces \(5\) after # in heading \[fixed\]
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 0
    And file "renamed.md" now has content
      """
      # Removed
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
    When executing "trident precommit --show=verbose"
    Then it prints the lines to STDERR
      """
      2 Markdown
      running 1 tasks
      """
    And it prints the block matching
      """
      fix Markdown \(rumdl\)
      .*rumdl(.exe)? fmt added.md$
      added.md:1:2: \[MD019\] Multiple spaces \(5\) after # in heading \[fixed\]
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

  Scenario: staged deletion of a file that is kept in the workspace
    Given I ran "git rm --cached --quiet removed.md"
    When executing "trident precommit --show=verbose"
    Then it prints to STDERR
      """
      1 Markdown
      """
    And it prints nothing to STDOUT
    And the exit code is 0
    And all files are unchanged
    And the staged changes are
      """
      diff --git a/removed.md b/removed.md
      deleted file mode 100644
      index 50032d8..0000000
      --- a/removed.md
      +++ /dev/null
      @@ -1 +0,0 @@
      -#     Removed
      """
