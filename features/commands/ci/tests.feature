Feature: CI runs all fixes, formatters, lints, and tests

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """

  Scenario:
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "command": "echo test one" },
          { "name": "unit tests", "command": "echo unit" }
        ]
      }
      """
    And a file "main.py" with content
      """
      print("hello")
      """
    When executing "trident ci --show=output"
    Then it prints the lines to STDERR
      """
      1 JSON, 1 Python, 1 other
      running 8 tools
      """
    And it prints the block
      """
      echo test one
      test one
      """
    And it prints the block
      """
      unit tests
      unit
      """
    And it prints the block
      """
      lint Python (ruff)
      All checks passed!
      """
    And the exit code is 0

  Scenario: test fails
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "failing test", "command": "tests/fail.sh" }
        ]
      }
      """
    And an executable file "tests/fail.sh" with content
      """
      #!/usr/bin/env bash
      echo "custom test failed"
      exit 4
      """
    When executing "trident ci --show=output"
    Then it prints the block
      """
      failing test
      custom test failed
      """
    And the exit code is 4

  Scenario: --test selects named tests
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo unit" },
          { "name": "cuke", "command": "echo cuke" },
          { "name": "slow", "command": "echo slow" }
        ]
      }
      """
    When executing "trident ci --test=unit+cuke --show=output"
    Then it prints the lines to STDERR
      """
      1 JSON, 1 other
      running 5 tools
      """
    And it prints the block
      """
      unit
      unit
      """
    And it prints the block
      """
      cuke
      cuke
      """
    And it does not print
      """
      slow
      """
    And the exit code is 0

  Scenario: commands.ci.test runs those tests by default
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo unit" },
          { "name": "cuke", "command": "echo cuke" }
        ],
        "commands": {
          "ci": {
            "test": ["unit"]
          }
        }
      }
      """
    When executing "trident ci --show=output"
    Then it prints the lines to STDERR
      """
      1 JSON, 1 other
      running 4 tools
      """
    And it prints the block
      """
      unit
      unit
      """
    And it does not print
      """
      cuke
      """
    And the exit code is 0

  Scenario: unknown test name
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo unit" },
          { "name": "cuke", "command": "echo cuke" }
        ]
      }
      """
    When executing "trident ci --test=unit+missing"
    Then it prints
      """
      unknown test: missing
      available tests: unit, cuke
      """
    And the exit code is 1
