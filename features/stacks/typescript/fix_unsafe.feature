Feature: unsafe-fix TypeScript

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """
    And a file "main.ts" with unix newlines
      """
      const name = "Alice";
      const greeting = "Hello, " + name + "!";
      """

  Scenario: fix-unsafe
    When executing "trident fix-unsafe --show=output"
    Then it prints the lines
      """
      unsafe-fix TypeScript (Biome)
      Checked 1 file in 3ms. Fixed 1 file.
      """
    And the exit code is 0
    And file "main.ts" now has content
      """
      const name = "Alice";
      const _greeting = `Hello, ${name}!`;
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: fix
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TypeScript (Biome)
      Formatted 1 file in 663µs. No fixes applied.
      """
    And the exit code is 0
    And file "main.ts" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
