Feature: pitstop --test runs selected tests

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """

  Scenario: without --test, configured tests do not run
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "prettier": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo unit" },
          { "name": "cuke", "command": "echo cuke" }
        ]
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines to STDERR
      """
      1 JSON, 1 other
      running 1 tools
      """
    And it does not print
      """
      unit
      """
    And it does not print
      """
      cuke
      """
    And the exit code is 0

  Scenario: commands.pitstop.test runs those tests by default
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "prettier": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo unit" },
          { "name": "cuke", "command": "echo cuke" }
        ],
        "commands": {
          "pitstop": {
            "test": ["unit"]
          }
        }
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines to STDERR
      """
      1 JSON, 1 other
      running 2 tools
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

  Scenario: --test overrides commands.pitstop.test
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "prettier": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo unit" },
          { "name": "cuke", "command": "echo cuke" }
        ],
        "commands": {
          "pitstop": {
            "test": ["unit"]
          }
        }
      }
      """
    When executing "trident pitstop --test=cuke --show=output"
    Then it prints the lines to STDERR
      """
      1 JSON, 1 other
      running 2 tools
      """
    And it prints the block
      """
      cuke
      cuke
      """
    And it does not print
      """
      unit
      """
    And the exit code is 0

  Scenario: --test selects named tests
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "prettier": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo unit" },
          { "name": "cuke", "command": "echo cuke" },
          { "name": "slow", "command": "echo slow" }
        ]
      }
      """
    When executing "trident pitstop --test=unit+cuke --show=output"
    Then it prints the lines to STDERR
      """
      1 JSON, 1 other
      running 3 tools
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

  Scenario: test fails
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "prettier": { "enabled": false }
        },
        "tests": [
          { "name": "failing", "command": "tests/fail.sh" }
        ]
      }
      """
    And an executable file "tests/fail.sh" with content
      """
      #!/usr/bin/env bash
      echo "custom test failed"
      exit 4
      """
    When executing "trident pitstop --test=failing --show=output"
    Then it prints the block
      """
      failing
      custom test failed
      """
    And the exit code is 4

  Scenario: unknown test name in commands.pitstop.test
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo unit" }
        ],
        "commands": {
          "pitstop": {
            "test": ["missing"]
          }
        }
      }
      """
    When executing "trident pitstop"
    Then it prints
      """
      unknown test: missing
      available tests: unit
      """
    And the exit code is 1

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
    When executing "trident pitstop --test=unit+missing"
    Then it prints
      """
      unknown test: missing
      available tests: unit, cuke
      """
    And the exit code is 1
