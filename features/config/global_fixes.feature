Feature: custom fixes

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """

  Scenario: custom fixes run in the order defined
    Given a file "trident.json" with content
      """
      {
        "global-fixes": [
          {
            "name": "my global fix 1",
            "command": "echo global fix 1 running"
          },
          {
            "name": "my global fix 2",
            "command": "echo global fix 2 running"
          }
        ]
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      delete empty folders
      my global fix 1
      global fix 1 running
      """
    And it prints the lines
      """
      my global fix 2
      global fix 2 running
      """
    And it prints the lines
      """
      fix JSON (Biome)
      """
    And the exit code is 0

  Scenario: custom lint fails
    Given a file "trident.json" with content
      """
      {
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
