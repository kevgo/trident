Feature: custom lints

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """

  Scenario: custom lint passes
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "global-lints": [
          { "command": "lints/one.sh" },
          {
            "name": "list all files",
            "command": "find . | sort | xargs echo"
          }
        ]
      }
      """
    And an executable file "lints/one.sh" with content
      """
      #!/usr/bin/env bash
      echo "custom lint is running"
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lints/one.sh
      custom lint is running
      """
    And it prints the block
      """
      list all files
      . ./lints ./lints/one.sh ./run-that-app ./trident.json
      """
    And the exit code is 0

  Scenario: custom lint fails
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "global-lints": [
          { "command": "lints/fail.sh" }
        ]
      }
      """
    And an executable file "lints/fail.sh" with content
      """
      #!/usr/bin/env bash
      echo "custom lint failed"
      exit 4
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lints/fail.sh
      custom lint failed
      """
    And the exit code is 4
