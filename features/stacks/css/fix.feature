Feature: fix CSS

  Background:
    Given a file "run-that-app" with unix newlines
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: valid CSS
    Given a file "main.css" with unix newlines
      """
      .foo {
      \tcolor: red;
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix CSS (Biome)
      """
    And the exit code is 0
    And file "main.css" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted CSS
    Given a file "main.css" with unix newlines
      """
      .foo {
        color : red ;
      }
      """
    And a file "other.css" with unix newlines
      """
      .bar {
        color : blue ;
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix CSS (Biome)
      """
    And the exit code is 0
    And file "main.css" now has content
      """
      .foo {
      \tcolor: red;
      }
      """
    And file "other.css" now has content
      """
      .bar {
      \tcolor: blue;
      }
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid CSS
    Given a file "main.css" with unix newlines
      """
      .foo {
        col
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix CSS (Biome)
      Found 2 errors.
      """
    And the exit code is 1
    And file "main.css" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
