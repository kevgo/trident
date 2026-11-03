Feature: "trident lint" chooses the smallest "--scope" based on the workspace state

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

  Scenario: on main branch, no uncommitted files
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown, 1 other
      running 2 tasks
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: on main branch, has uncommitted files
    And a file "untracked.md" with content
      """
      #     Untracked
      """
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
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

  Scenario: on feature branch, branch has no commits, no uncommitted files,
    Given I ran "git checkout -b feature"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown, 1 other
      running 2 tasks
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: on feature branch, branch has commits, no uncommitted files,
    Given I ran "git checkout -b feature"
    And a committed file "committed-on-branch.md" with content
      """
      #     Branch
      """
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      committed-on-branch.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: on feature branch, branch has commits, has uncommitted files
    Given I ran "git checkout -b feature"
    And a committed file "committed-on-branch.md" with content
      """
      #     Branch
      """
    And a file "untracked.md" with content
      """
      #     Untracked
      """
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      untracked.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it prints the block
      """
      committed-on-branch.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: on main branch, only uncommitted deletions
    Given a committed file "deleted.md" with content
      """
      #     Deleted
      """
    And I ran "git rm deleted.md"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      0 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And file "on-main.md" is unchanged

  Scenario: on feature branch, branch only deletes files
    Given a committed file "deleted.md" with content
      """
      #     Deleted
      """
    And I ran "git checkout -b feature"
    And I ran "git rm deleted.md"
    And I ran "git commit --message=delete"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      0 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And file "on-main.md" is unchanged

  Scenario: on feature branch, branch has commits, only uncommitted deletions
    Given a committed file "deleted.md" with content
      """
      #     Deleted
      """
    And I ran "git checkout -b feature"
    And a committed file "committed-on-branch.md" with content
      """
      #     Deleted
      """
    And I ran "git rm deleted.md"
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      0 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      committed-on-branch.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And file "committed-on-branch.md" is unchanged
    And file "on-main.md" is unchanged

  Scenario: deleted files are not passed to tools that process individual files
    Given a committed file "deleted.md" with content
      """
      # Deleted
      """
    And I ran "git rm deleted.md"
    And a file "untracked.md" with content
      """
      #     Untracked
      """
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 2 tasks
      """
    And it prints the block
      """
      untracked.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And it does not print
      """
      deleted.md
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1

  Scenario: stacks without changes don't get linted
    Given a file "untracked.rs" with content
      """
      // Rust code
      """
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 Rust
      running 1 tasks
      """
    And it does not print
      """
      lint Markdown
      """
    And it does not print
      """
      on-main.md
      """
    And the exit code is 0
    And all files are unchanged
