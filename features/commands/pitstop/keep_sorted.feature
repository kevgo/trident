Feature: "trident pitstop" keep-sorted only rewrites files changed on the current branch

  Background:
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      keep-sorted 0.9.1
      ripgrep 15.2.0
      taplo 0.10.0
      """
    And a committed file "trident.json" with content
      """
      {
        "applications": {
          "keep-sorted": {
            "enabled": true
          }
        }
      }
      """
    And a committed file "untouched.toml" with content
      """
      # keep-sorted start
      b = 1
      a = 1
      # keep-sorted end
      """

  Scenario: on a feature branch, does not sort files that were not changed on the branch
    Given I ran "git checkout -b feature"
    And a file "changed.toml" with content
      """
      # keep-sorted start
      b = 2
      a = 2
      # keep-sorted end
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      fix TOML (keep-sorted)
      """
    And the exit code is 0
    And file "changed.toml" now has content
      """
      # keep-sorted start
      a = 2
      b = 2
      # keep-sorted end
      """
    And file "untouched.toml" is unchanged

  Scenario: on the main branch, does not sort committed files that were not changed
    Given a committed file "changed.toml" with content
      """
      # keep-sorted start
      b = 3
      a = 3
      # keep-sorted end
      """
    And I change file "changed.toml" to
      """
      # keep-sorted start
      b = 4
      a = 4
      # keep-sorted end
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      fix TOML (keep-sorted)
      """
    And the exit code is 0
    And file "changed.toml" now has content
      """
      # keep-sorted start
      a = 4
      b = 4
      # keep-sorted end
      """
    And file "untouched.toml" is unchanged
