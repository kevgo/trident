Feature: fix TypeScript

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: valid TypeScript
    Given a file "main.ts" with unix newlines
      """
      console.log("hello");
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      TypeScript (Biome)
      """
    And the exit code is 0
    And file "main.ts" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted TypeScript
    Given a file "main.ts" with unix newlines
      """
      console.log(  "hello"  );
      """
    And a file "other.ts" with unix newlines
      """
      console.log(  "other"  );
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TypeScript (Biome)
      Formatted 2 files in 558µs. Fixed 2 files.
      """
    And the exit code is 0
    And file "main.ts" now has content
      """
      console.log("hello");
      """
    And file "other.ts" now has content
      """
      console.log("other");
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid TypeScript
    Given a file "main.ts" with unix newlines
      """
      console.log("
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TypeScript (Biome)
      Formatted 1 file in 261µs. No fixes applied.
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × unterminated string literal

        > 1 │ console.log("
            │             ^
          2 │

        i

        > 1 │ console.log("
            │             ^
          2 │

        i The closing quote must be on the same line.


      main.ts:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `)` but instead the file ends

          1 │ console.log("
        > 2 │
            │

        i the file ends here

          1 │ console.log("
        > 2 │
            │


      main.ts format ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Code formatting aborted due to parsing errors. To format code with errors, enable the 'formatter.formatWithErrors' option.


      format ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × No files were processed in the specified paths.

        i Check your biome.json or biome.jsonc to ensure the paths are not ignored by the configuration.

        i These paths were provided but ignored:

        - main.ts


      """
    And the exit code is 1
    And file "main.ts" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
