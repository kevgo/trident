Feature: pitstop TypeScript

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: unformatted TypeScript
    Given a file "main.ts" with content
      """
      console.log(  "hello"  );
      """
    And a file "other.ts" with content
      """
      console.log(  "other"  );
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix TypeScript (Biome)
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

  Scenario: unformatted TypeScript with lint errors
    Given a file "main.ts" with content
      """
      function foo() {
        let a = 1;
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix TypeScript (Biome)
      lint TypeScript (Biome)
      Found 3 warnings.
      """
    And the exit code is 0
    And file "main.ts" now has content
      """
      function foo() {
      \tlet a = 1;
      }
      """
