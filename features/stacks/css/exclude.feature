Feature: ignore a CSS file

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """
    And a file "trident.json" with unix newlines
      """
      {
        "ignore-files": ["bad.css"]
      }
      """

  Scenario: linting
    Given a file "good.css" with unix newlines
      """
      .foo {
        color: green;
      }
      """
    And a file "bad.css" with unix newlines
      """
      .bar {
        col
      }
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint JSON (Biome)
      Checked 1 file in 631µs. No fixes applied.
      lint CSS (Biome)
      Checked 1 file in 703µs. No fixes applied.
      """
    And the exit code is 0
    And file "bad.css" is unchanged
    And file "good.css" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: fixing
    Given a file "good.css" with unix newlines
      """
      .foo {
        color : red ;
      }
      """
    And a file "bad.css" with unix newlines
      """
      .bar {
        color : green ;
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix JSON (Biome)
      Formatted 1 file in 308µs. Fixed 1 file.
      fix CSS (Biome)
      Formatted 1 file in 489µs. Fixed 1 file.
      """
    And the exit code is 0
    And file "good.css" now has content
      """
      .foo {
      \tcolor: red;
      }
      """
    And file "bad.css" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
