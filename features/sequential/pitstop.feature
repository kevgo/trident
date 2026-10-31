Feature: run a pitstop sequentially

  @this
  Scenario: custom fixes, lints, and tests
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "delete-empty-folders": { "enabled": false },
          "biome": { "enabled": false }
        },
        "global-fixes": [
          { "name": "my fix", "command": "echo my fix done" }
        ],
        "global-lints": [
          { "name": "slow lint", "command": "sleep 0.3 && echo slow lint done" },
          { "name": "fast lint", "command": "echo fast lint done" }
        ],
        "tests": [
          { "name": "unit", "command": "echo unit done" }
        ]
      }
      """
    When executing "trident pitstop --sequential --test=unit --show=output"
    Then it prints
      """
      fix all (my fix)
      my fix done
      lint all (slow lint)
      slow lint done
      lint all (fast lint)
      fast lint done
      test all (unit)
      unit done
      """
    And the exit code is 0
