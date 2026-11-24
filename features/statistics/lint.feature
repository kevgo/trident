Feature: display statistics about the codebase

  Scenario: multiple stacks
    Given a file "run-that-app" with unix newlines
      """
      biome 2.4.0
      ruff 0.15.16
      """
    And a file "main.py" with unix newlines
      """
      print("Hello, world!")
      """
    And a file "main.css" with unix newlines
      """
      p {
      \tcolor: red;
      }
      """
    And a file "main.ts" with unix newlines
      """
      console.log("hello");
      """
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 3 tasks
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
