Feature: lint TypeScript

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
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint TypeScript (Biome)
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
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint TypeScript (Biome)
      """
    And the exit code is 0
    And file "main.ts" is unchanged
    And file "other.ts" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid TypeScript
    Given a file "main.ts" with unix newlines
      """
      console.log("
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint TypeScript (Biome)
      Checked 1 file in 1021µs. No fixes applied.
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


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      """
    And the exit code is 1
    And file "main.ts" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
