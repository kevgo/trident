Feature: fix Markdown with Tikibase

  Background:
    Given a file "run-that-app" with content
      """
      biome 0.21.0
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      tikibase 0.6.2
      """
    And a file "tikibase.json" with content
      """
      {
        "ignore": [
          "run-that-app"
        ],
        "bidiLinks": true
      }
      """

  Scenario: valid Markdown
    Given a file "one.md" with content
      """
      # One

      also check out [Two](two.md)
      """
    And a file "two.md" with content
      """
      # Two

      also check out [One](one.md)
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Markdown (tikibase)
      fix Markdown (rumdl)
      """
    And the exit code is 0
    And file "one.md" is unchanged
    And file "two.md" is unchanged

  Scenario: unformatted Markdown
    Given a file "one.md" with content
      """
      # One

      also check out [Two](two.md)
      """
    And a file "two.md" with content
      """
      # Two
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Markdown (tikibase)
      two.md:3  added one.md to occurrences section
      fix Markdown (rumdl)
      """
    And the exit code is 0
    And file "one.md" is unchanged
    And file "two.md" now has content
      """
      # Two

      ## occurrences

      - [One](one.md)
      """

  Scenario: invalid Markdown
    Given a file "main.md" with content
      """
      # hello

      [e]()
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      fix Markdown (tikibase)
      """
    And it prints the block
      """
      fix Markdown (rumdl)
      main.md:3:1: [MD042] Empty link found: [e]()
      """
    And the exit code is 0
    And file "main.md" is unchanged
