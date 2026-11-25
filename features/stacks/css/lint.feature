Feature: lint CSS

  Background:
    Given a file "run-that-app" with content
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
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint CSS (Biome)
      Checked 1 file in 819µs. No fixes applied.
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
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint CSS (Biome)
      """
    And the exit code is 0
    And file "main.css" is unchanged
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
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint CSS (Biome)
      Checked 1 file in 767µs. No fixes applied.
      Found 2 errors.
      main.css:3:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Expected a compound selector but instead found '}'.

          1 │ .foo {
          2 │   col
        > 3 │ }
            │ ^
          4 │

        i Expected a compound selector here.

          1 │ .foo {
          2 │   col
        > 3 │ }
            │ ^
          4 │


      main.css:4:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `{` but instead the file ends

          2 │   col
          3 │ }
        > 4 │
            │

        i the file ends here

          2 │   col
          3 │ }
        > 4 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      """
    And the exit code is 1
    And file "main.css" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
