Feature: lint TypeScript

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: valid TypeScript
    Given a file "main.ts" with content
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

  Scenario: unformatted TypeScript
    Given a file "main.ts" with content
      """
      console.log(  "hello"  );
      """
    And a file "other.ts" with content
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

  Scenario: invalid TypeScript
    Given a file "main.ts" with content
      """
      console.log("
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint TypeScript (Biome)
      Found 2 errors.
      """
    And the exit code is 1
    And file "main.ts" is unchanged
