Feature: ignore a CSS file

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """
    And a file "trident.json" with content
      """
      {
        "ignore-files": ["bad.css"]
      }
      """

  Scenario: linting
    Given a file "good.css" with content
      """
      .foo {
        color: green;
      }
      """
    And a file "bad.css" with content
      """
      .bar {
        col
      }
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint CSS (Biome)
      """
    And the exit code is 0
    And file "good.css" is unchanged
    And file "bad.css" is unchanged

  Scenario: fixing
    Given a file "good.css" with content
      """
      .foo {
        color : red ;
      }
      """
    And a file "bad.css" with content
      """
      .bar {
        color : green ;
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix CSS (Biome)
      """
    And the exit code is 0
    And file "good.css" now has content
      """
      .foo {
      \tcolor: red;
      }
      """
    And file "bad.css" is unchanged
