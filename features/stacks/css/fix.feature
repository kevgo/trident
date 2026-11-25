Feature: fix CSS

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
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix CSS (Biome)
      Formatted 1 file in 280µs. No fixes applied.
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
      fix all (delete empty folders)
      fix CSS (Biome)
      Formatted 2 files in 732µs. Fixed 2 files.
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
      fix all (delete empty folders)
      fix CSS (Biome)
      Formatted 1 file in 275µs. No fixes applied.
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


      main.css format ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Code formatting aborted due to parsing errors. To format code with errors, enable the 'formatter.formatWithErrors' option.


      format ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × No files were processed in the specified paths.

        i Check your biome.json or biome.jsonc to ensure the paths are not ignored by the configuration.

        i These paths were provided but ignored:

        - main.css


      """
    And the exit code is 1
    And file "main.css" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
