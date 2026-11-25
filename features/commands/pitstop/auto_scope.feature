Feature: "trident pitstop" chooses the smallest "--scope" based on the workspace state

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
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 Markdown, 1 other
      running 4 tasks
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      """
    And the exit code is 0
    And file "on-main.md" now has content
      """
      # Main
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames --untracked-files=all
      git town config get-parent
      git symbolic-ref --short refs/remotes/origin/HEAD
      git rev-parse --verify --quiet refs/heads/main
      git merge-base HEAD main
      git diff -z --name-only --no-renames {SHA}...HEAD
      git diff HEAD --check
      """

  Scenario: on main branch, has uncommitted files
    And a file "untracked.md" with content
      """
      #     Untracked
      """
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 4 tasks
      """
    And it prints the block
      """
      untracked.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      """
    And it does not print
      """
      on-main.md
      """
    And the exit code is 0
    And file "untracked.md" now has content
      """
      # Untracked
      """
    And file "on-main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames --untracked-files=all
      git diff HEAD --check
      """

  Scenario: on feature branch, branch has no commits, no uncommitted files,
    Given I ran "git checkout -b feature"
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 Markdown, 1 other
      running 4 tasks
      """
    And it prints the block
      """
      on-main.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      """
    And the exit code is 0
    And file "on-main.md" now has content
      """
      # Main
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames --untracked-files=all
      git town config get-parent
      git symbolic-ref --short refs/remotes/origin/HEAD
      git rev-parse --verify --quiet refs/heads/main
      git merge-base HEAD main
      git diff -z --name-only --no-renames {SHA}...HEAD
      git diff HEAD --check
      """

  Scenario: on feature branch, branch has commits, no uncommitted files,
    Given I ran "git checkout -b feature"
    And a committed file "committed-on-branch.md" with content
      """
      #     Branch
      """
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 4 tasks
      """
    And it prints the block
      """
      committed-on-branch.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      """
    And it does not print
      """
      on-main.md
      """
    And the exit code is 0
    And file "committed-on-branch.md" now has content
      """
      # Branch
      """
    And file "on-main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames --untracked-files=all
      git town config get-parent
      git symbolic-ref --short refs/remotes/origin/HEAD
      git rev-parse --verify --quiet refs/heads/main
      git merge-base HEAD main
      git diff -z --name-only --no-renames {SHA}...HEAD
      git diff HEAD --check
      """

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
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 Markdown
      running 4 tasks
      """
    And it prints the block
      """
      untracked.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      """
    And it does not print
      """
      committed-on-branch.md
      """
    And it does not print
      """
      on-main.md
      """
    And the exit code is 0
    And file "untracked.md" now has content
      """
      # Untracked
      """
    And file "committed-on-branch.md" is unchanged
    And file "on-main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames --untracked-files=all
      git diff HEAD --check
      """
