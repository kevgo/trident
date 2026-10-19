Feature: stack-specific fixes

  Background:
    Given a file "run-that-app" with content
      """
      biome 0.21.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """

  Scenario: "additional-fixes" adds custom fixes to the built-in ones for that stack
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "python": {
            "fix": {
              "add": [
                { "name": "my fix", "command": "echo MY FIX RAN" }
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
    When executing "trident fix --show=output"
    Then it prints the block
      """
      fix Python (ruff)
      """
    And it prints the block
      """
      fix Python (my fix)
      MY FIX RAN
      """
    And the exit code is 0

  Scenario: "replace-fixes" replaces the built-in fixes for that stack
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "python": {
            "fix": {
              "replace": [
                { "name": "my fix", "command": "echo MY FIX RAN" }
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
    When executing "trident fix --show=output"
    Then it prints the block
      """
      fix Python (my fix)
      MY FIX RAN
      """
    And it does not print any of these lines
      """
      fix Python (ruff)
      """
    And the exit code is 0

  Scenario: disable a stack's fixes
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "python": {
            "fix": {
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
    When executing "trident fix --show=output"
    Then it does not print any of these lines
      """
      fix Python (ruff)
      """
    And the exit code is 0

  Scenario: runs only when files of that stack exist
    Given a file "trident.json" with content
      """
      {
        "stacks": {
          "python": {
            "fix": {
              "add": [
                { "name": "my fix", "command": "echo MY FIX RAN" }
              ]
            }
          }
        }
      }
      """
    When executing "trident fix --show=output"
    Then it does not print any of these lines
      """
      my fix
      MY FIX RAN
      """
    And the exit code is 0
