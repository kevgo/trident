Feature: run lints sequentially

  Scenario: custom lints
    Given a file "trident.json" with content
      """
      {
        "global-lints": [
          { "name": "slow", "command": "sleep 0.3 && echo slow done" },
          { "name": "fast", "command": "echo fast done" }
        ]
      }
      """
    When executing "trident lint --sequential --show=output"
    Then it prints
      """
      slow
      slow done
      fast
      fast done
      """
    And the exit code is 0
