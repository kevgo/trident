Feature: "trident postedit" enables linters whose config is already committed

  Scenario: runs Tikibase even though its config file is not changed
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
    And I change file "one.md" to
      """
      # New one

      also check out [Two](two.md)
      """
    When executing "trident postedit --show=output"
    Then it prints the block
      """
      lint Markdown (tikibase)
      """
    And it prints the block
      """
      lint Markdown (rumdl)
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --untracked-files=all
      git diff HEAD --check
      """
