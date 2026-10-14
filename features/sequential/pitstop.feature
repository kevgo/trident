Feature: run a pitstop sequentially

  Scenario: custom fixes, lints, and tests
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "delete-empty-folders": { "enabled": false },
          "prettier": { "enabled": false }
        },
        "global-fixes": [
          { "name": "fix", "command": "echo fix done" }
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
      fix
      fix done
      slow lint
      slow lint done
      fast lint
      fast lint done
      unit
      unit done
      """
    And the exit code is 0
