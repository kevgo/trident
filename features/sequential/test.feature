Feature: run tests sequentially

  Background:
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "slow", "command": "sleep 0.3 && echo slow done" },
          { "name": "fast", "command": "echo fast done" }
        ]
      }
      """

  Scenario: without --sequential
    When executing "trident test --show=output"
    Then it prints
      """
      fast
      fast done
      slow
      slow done
      """
    And the exit code is 0

  Scenario: with --sequential
    When executing "trident test --sequential --show=output"
    Then it prints
      """
      slow
      slow done
      fast
      fast done
      """
    And the exit code is 0
