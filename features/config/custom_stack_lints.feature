Feature: stack-specific lints

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """

  Scenario: "additional-lints" adds custom lints to the built-in ones for that stack
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "python": {
            "lint": {
              "add": [
                { "name": "my lint", "command": "echo MY LINT RAN" }
              ]
            }
          }
        }
      }
      """
    And a file "main.py" with content
      """
      # some Python code
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Python (ruff)
      """
    And it prints the block
      """
      lint Python (my lint)
      MY LINT RAN
      """
    And the exit code is 0

  Scenario: "replace-lints" replaces the built-in lints for that stack
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "python": {
            "lint": {
              "replace": [
                { "name": "my lint", "command": "echo MY LINT RAN" }
              ]
            }
          }
        }
      }
      """
    And a file "main.py" with content
      """
      # some Python code
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Python (my lint)
      MY LINT RAN
      """
    And it does not print any of these lines
      """
      lint Python (ruff)
      """
    And the exit code is 0

  Scenario: disable a stack's lints
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "python": {
            "lint": {
              "replace": []
            }
          }
        }
      }
      """
    And a file "main.py" with content
      """
      # some Python code
      """
    When executing "trident lint --show=output"
    Then it does not print any of these lines
      """
      lint Python (ruff)
      """
    And the exit code is 0

  Scenario: runs only when files of that stack exist
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "python": {
            "lint": {
              "add": [
                { "name": "my lint", "command": "echo MY LINT RAN" }
              ]
            }
          }
        }
      }
      """
    When executing "trident lint --show=output"
    Then it does not print any of these lines
      """
      my lint
      MY LINT RAN
      """
    And the exit code is 0
