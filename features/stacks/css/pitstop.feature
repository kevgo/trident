Feature: pitstop CSS

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: unformatted CSS
    Given a file "main.css" with content
      """
      .foo {
        color : red ;
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix CSS (Biome)
      lint CSS (Biome)
      """
    And the exit code is 0
    And file "main.css" now has content
      """
      .foo {
      \tcolor: red;
      }
      """

  Scenario: unformatted CSS with lint error
    Given a file "main.css" with content
      """
      .foo {
        colr: blue;
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix CSS (Biome)
      lint CSS (Biome)
      Found 1 error.
        × Unknown property is not allowed.
      """
    And the exit code is 1
    And file "main.css" now has content
      """
      .foo {
      \tcolr: blue;
      }
      """
