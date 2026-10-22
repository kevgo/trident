Feature: keep-sorted support

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      keep-sorted 0.9.1
      ripgrep 15.2.0
      taplo 0.10.0
      """
    And a file "unsorted_1.toml" with content
      """
      # keep-sorted start
      a = 1
      b = 1
      # keep-sorted end
      """
    And a file "unsorted_2.toml" with content
      """
      # keep-sorted start
      c = 1
      d = 1
      # keep-sorted end
      """

  Scenario: keep-sorted is disabled by default
    When executing "trident fix --show=output"
    Then it does not print
      """
      keep-sorted
      """
    And file "unsorted_1.toml" is unchanged
    And file "unsorted_2.toml" is unchanged

  Scenario: keep-sorted sorts marker-bearing files when enabled
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "keep-sorted": {
            "enabled": true
          }
        }
      }
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      sort TOML (keep-sorted)
      """
    And the exit code is 0
    And file "unsorted_1.toml" now has content
      """
      # keep-sorted start
      a = 1
      b = 1
      # keep-sorted end
      """
    And file "unsorted_2.toml" now has content
      """
      # keep-sorted start
      c = 1
      d = 1
      # keep-sorted end
      """

  Scenario: does not sort globally ignored files
    Given a file "trident.json" with content
      """
      {
        "ignore-files": ["unsorted_1.toml"],
        "applications": {
          "keep-sorted": {
            "enabled": true
          }
        }
      }
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      keep-sorted
      """
    And file "unsorted_1.toml" is unchanged
    And file "unsorted_2.toml" now has content
      """
      # keep-sorted start
      c = 1
      d = 1
      # keep-sorted end
      """

  Scenario: does not sort files that should not be sorted
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "keep-sorted": {
            "enabled": true,
            "ignore-files": ["unsorted_1.toml"]
          }
        }
      }
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      keep-sorted
      """
    And file "unsorted_1.toml" is unchanged
    And file "unsorted_2.toml" now has content
      """
      # keep-sorted start
      c = 1
      d = 1
      # keep-sorted end
      """
