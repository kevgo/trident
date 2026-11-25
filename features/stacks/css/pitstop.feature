Feature: pitstop CSS

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: unformatted CSS
    Given a file "main.css" with unix newlines
      """
      .foo {
        color : red ;
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix CSS (Biome)
      Formatted 1 file in 768µs. Fixed 1 file.
      lint CSS (Biome)
      Checked 1 file in 6ms. No fixes applied.
      """
    And the exit code is 0
    And file "main.css" now has content
      """
      .foo {
      \tcolor: red;
      }
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted CSS with lint error
    Given a file "main.css" with unix newlines
      """
      .foo {
        colr: blue;
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix CSS (Biome)
      Formatted 1 file in 381µs. Fixed 1 file.
      lint CSS (Biome)
      Checked 1 file in 660µs. No fixes applied.
      Found 1 error.
      main.css:2:2 lint/correctness/noUnknownProperty ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Unknown property is not allowed.

          1 │ .foo {
        > 2 │ colr: blue;
            │ ^^^^
          3 │ }
          4 │

        i See CSS Specifications and browser specific properties for more details.

        i To resolve this issue, replace the unknown property with a valid CSS property.


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      """
    And the exit code is 1
    And file "main.css" now has content
      """
      .foo {
      \tcolr: blue;
      }
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
