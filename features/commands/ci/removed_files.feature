Feature: "trident ci" processes deleted or renamed files

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
    And a committed file "unrelated.md" with content
      """
      #     Unrelated
      """
    And a committed file "removed.md" with content
      """
      # Removed
      """

  Scenario: --scope=uncommitted, uncommitted deletion
    Given I delete file "removed.md"
    When executing "trident ci --scope=uncommitted --show=output"
    Then it prints the lines to STDERR
      """
      1 Markdown
      running 1 tasks
      """
    And it prints the block
      """
      lint Markdown (rumdl)
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      fix Markdown
      """
    And the exit code is 1
    And all files are unchanged

  @this
  Scenario: --scope=branch, committed deletion
    Given I ran "git checkout -b feature"
    And I delete file "removed.md"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident ci --scope=branch --show=output"
    Then it prints the lines to STDERR
      """
      1 Markdown
      running 1 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      fix Markdown
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: --scope=uncommitted with an uncommitted rename
    Given I rename file "removed.md" to "renamed.md"
    When executing "trident ci --scope=uncommitted --show=output"
    Then it prints the lines to STDERR
      """
      2 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      unrelated.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged
