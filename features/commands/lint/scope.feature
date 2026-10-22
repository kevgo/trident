Feature: "trident lint --scope" selects which files to lint

  Background:
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """
    And a committed file "on-main.md" with content
      """
      #     Main
      """
    And I ran "git checkout -b feature"
    And a committed file "committed-on-branch.md" with content
      """
      #     Branch
      """
    And a file "untracked.md" with content
      """
      #     Untracked
      """

  Scenario: --scope=uncommitted lints only uncommitted files
    When executing "trident lint --scope=uncommitted --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tools
      """
    And it prints the lines
      """
      lint Markdown (rumdl)
      """
    And it prints the block
      """
      untracked.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      committed-on-branch.md
      """
    And it does not print
      """
      on-main.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: --scope=branch lints files changed on the current branch
    When executing "trident lint --scope=branch --show=output"
    Then it prints to STDERR
      """
      2 Markdown
      running 2 tools
      """
    And it prints the block
      """
      committed-on-branch.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it prints the block
      """
      untracked.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      on-main.md
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: --scope=all lints all files
    When executing "trident lint --scope=all --show=output"
    Then it prints to STDERR
      """
      3 Markdown, 1 other
      running 2 tools
      """
    And it prints the block
      """
      committed-on-branch.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it prints the block
      """
      untracked.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged
