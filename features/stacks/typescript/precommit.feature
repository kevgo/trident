Feature: precommit TypeScript

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: valid TypeScript
    Given a file "main.ts" with unix newlines
      """
      console.log("hello");
      """
    And I ran "git add main.ts"
    When executing "trident precommit --show=output"
    Then it prints the block
      """
      fix TypeScript (Biome)
      """
    And the exit code is 0
    And file "main.ts" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
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
    And I ran "git add main.ts other.ts"
    When executing "trident precommit --show=output"
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
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      git add -- main.ts other.ts
      """

  Scenario: invalid TypeScript
    Given a file "main.ts" with unix newlines
      """
      console.log("
      """
    And I ran "git add main.ts"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix TypeScript (Biome)
      Found 2 errors.
      """
    And the exit code is 0
    And file "main.ts" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """
