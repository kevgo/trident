Feature: lint CSS

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: valid CSS
    Given a file "main.css" with content
      """
      .foo {
      \tcolor: red;
      }
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint CSS (Biome)
      """
    And the exit code is 0
    And file "main.css" is unchanged

  Scenario: unformatted CSS
    Given a file "main.css" with content
      """
      .foo {
        color : red ;
      }
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint CSS (Biome)
      """
    And the exit code is 0
    And file "main.css" is unchanged

  Scenario: invalid CSS
    Given a file "main.css" with content
      """
      .foo {
        col
      }
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint CSS (Biome)
      Found 2 errors.
      """
    And the exit code is 1
    And file "main.css" is unchanged
