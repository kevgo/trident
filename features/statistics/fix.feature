Feature: display statistics about the codebase when fixing

  Scenario: multiple stacks
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """
    And a file "main.py" with unix newlines
      """
      print("Hello, world!")
      """
    And a file "main.css" with unix newlines
      """
      p {
        color: red;
      }
      """
    And a file "main.ts" with unix newlines
      """
      console.log("hello");
      """
    When executing "trident fix --show=output"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 5 tasks
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
