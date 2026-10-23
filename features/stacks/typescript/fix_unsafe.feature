Feature: unsafe-fix TypeScript

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """
    And a file "main.ts" with content
      """
      const name = "Alice";
      const greeting = "Hello, " + name + "!";
      """

  Scenario: fix-unsafe
    When executing "trident fix-unsafe --show=output"
    Then it prints the lines
      """
      unsafe-fix TypeScript (Biome)
      """
    And the exit code is 0
    And file "main.ts" now has content
      """
      const name = "Alice";
      const _greeting = `Hello, ${name}!`;
      """

  Scenario: fix
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix TypeScript (Biome)
      """
    And the exit code is 0
    And file "main.ts" is unchanged
