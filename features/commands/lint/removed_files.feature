Feature: "trident lint" lints all files of a stack in which files were deleted or renamed

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

  Scenario: unstaged deletion
    Given I delete file "removed.md"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: staged deletion
    Given I delete file "removed.md"
    And I ran "git add -A"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: unstaged rename
    Given I rename file "removed.md" to "renamed.md"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
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
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
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
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: deletion committed on the feature branch
    Given I ran "git checkout -b feature"
    And I delete file "removed.md"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: rename committed on the feature branch
    Given I ran "git checkout -b feature"
    And I rename file "removed.md" to "renamed.md"
    And I ran "git add -A"
    And I ran "git commit -m rename"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      removed.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: --scope=branch sees uncommitted deletions
    Given I ran "git checkout -b feature"
    And I delete file "removed.md"
    When executing "trident lint --scope=branch --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: --scope=uncommitted does not see deletions committed on the feature branch
    Given I ran "git checkout -b feature"
    And I delete file "removed.md"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident lint --scope=uncommitted --show=output"
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

  Scenario: deleting a file in another stack does not lint this stack
    Given a committed file "removed.txt" with content
      """
      text
      """
    And I delete file "removed.txt"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 other
      running 1 tasks
      """
    And it does not print
      """
      unrelated.md
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: deleting an ignored file does not lint its stack
    Given a committed file "trident.jsonc" with content
      """
      {
        "ignore-files": ["removed.md"]
      }
      """
    And I delete file "removed.md"
    When executing "trident lint --show=output"
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

  Scenario: the file count shows changed files, not linted files
    Given a committed file "also-unrelated.md" with content
      """
      # Also unrelated
      """
    And I delete file "removed.md"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged
