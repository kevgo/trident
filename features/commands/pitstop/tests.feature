Feature: pitstop --test runs selected tests

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      shellcheck 0.11.0
      shfmt 3.14.1
      """

  Scenario: without --test, configured tests do not run
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "cuke", "command": "echo running cuke tests" }
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
      running unit tests
      """
    And it does not print
      """
      running cuke tests
      """
    And the exit code is 0

  Scenario: commands.pitstop.test runs those tests by default
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "cuke", "command": "echo running cuke tests" }
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
      test all (unit)
      running unit tests
      """
    And it does not print
      """
      running cuke tests
      """
    And the exit code is 0

  Scenario: --test overrides commands.pitstop.test
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "cuke", "command": "echo running cuke tests" }
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
      test all (cuke)
      running cuke tests
      """
    And it does not print
      """
      running unit tests
      """
    And the exit code is 0

  Scenario: --test selects named tests
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "cuke", "command": "echo running cuke tests" },
          { "name": "slow", "command": "echo running slow tests" }
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
      test all (unit)
      running unit tests
      """
    And it prints the block
      """
      test all (cuke)
      running cuke tests
      """
    And it does not print
      """
      running slow tests
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
      test all (failing)
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
