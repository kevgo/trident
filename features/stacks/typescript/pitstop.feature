Feature: pitstop TypeScript

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
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
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TypeScript (Biome)
      Formatted 2 files in 1234µs. Fixed 2 files.
      lint TypeScript (Biome)
      Checked 2 files in 1305µs. No fixes applied.
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

  Scenario: unformatted TypeScript with lint errors
    Given a file "main.ts" with unix newlines
      """
      function foo() {
        let a = 1;
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TypeScript (Biome)
      Formatted 1 file in 384µs. Fixed 1 file.
      lint TypeScript (Biome)
      Checked 1 file in 2ms. No fixes applied.
      Found 3 warnings.
      main.ts:1:10 lint/correctness/noUnusedVariables  FIXABLE  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        ! This function foo is unused.

        > 1 │ function foo() {
            │          ^^^
          2 │ let a = 1;
          3 │ }

        i Unused variables are often the result of typos, incomplete refactors, or other sources of bugs.

        i Unsafe fix: If this is intentional, prepend foo with an underscore.

          1   │ - function·foo()·{
            1 │ + function·_foo()·{
          2 2 │   let a = 1;
          3 3 │   }


      main.ts:2:2 lint/style/useConst  FIXABLE  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        ! This let declares a variable that is only assigned once.

          1 │ function foo() {
        > 2 │ let a = 1;
            │ ^^^
          3 │ }
          4 │

        i 'a' is never reassigned.

          1 │ function foo() {
        > 2 │ let a = 1;
            │     ^
          3 │ }
          4 │

        i Safe fix: Use const instead.

          1 1 │   function foo() {
          2   │ - → let·a·=·1;
            2 │ + → const·a·=·1;
          3 3 │   }
          4 4 │


      main.ts:2:6 lint/correctness/noUnusedVariables  FIXABLE  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        ! This variable a is unused.

          1 │ function foo() {
        > 2 │ let a = 1;
            │     ^
          3 │ }
          4 │

        i Unused variables are often the result of typos, incomplete refactors, or other sources of bugs.

        i Unsafe fix: If this is intentional, prepend a with an underscore.

          1 1 │   function foo() {
          2   │ - → let·a·=·1;
            2 │ + → let·_a·=·1;
          3 3 │   }
          4 4 │


      """
    And the exit code is 0
    And file "main.ts" now has content
      """
      function foo() {
      \tlet a = 1;
      }
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
