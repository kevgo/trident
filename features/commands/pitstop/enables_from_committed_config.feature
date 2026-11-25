Feature: "trident pitstop" enables fixes whose config is already committed

  Background:
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      tikibase 0.6.2
      """
    And a committed file "tikibase.json" with content
      """
      {
        "ignore": [
          "run-that-app"
        ],
        "bidiLinks": true
      }
      """
    And a committed file "one.md" with content
      """
      # One

      also check out [Two](two.md)
      """
    And a committed file "two.md" with content
      """
      # Two

      also check out [One](one.md)
      """

  Scenario: on a feature branch, runs Tikibase even though its config file is not changed
    Given I ran "git checkout -b feature"
    And I change file "one.md" to
      """
      #     One

      also check out [Two](two.md)
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      fix Markdown (tikibase)
      """
    And it prints the block
      """
      fix Markdown (rumdl)
      """
    And the exit code is 0
    And file "one.md" now has content
      """
      # One

      also check out [Two](two.md)
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames --untracked-files=all
      git diff HEAD --check
      """

  Scenario: on the main branch, runs Tikibase even though its config file is not changed
    Given I change file "one.md" to
      """
      #     One

      also check out [Two](two.md)
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      fix Markdown (tikibase)
      """
    And it prints the block
      """
      fix Markdown (rumdl)
      """
    And the exit code is 0
    And file "one.md" now has content
      """
      # One

      also check out [Two](two.md)
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames --untracked-files=all
      git diff HEAD --check
      """
