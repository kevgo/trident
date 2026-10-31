Feature: CI runs all fixes, formatters, lints, and tests

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      shellcheck 0.11.0
      shfmt 3.14.1
      """

  Scenario:
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "command": "echo test one" },
          { "name": "unit", "command": "echo running unit tests" }
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
      test all (echo test one)
      test one
      """
    And it prints the block
      """
      test all (unit)
      running unit tests
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
      test all (failing test)
      custom test failed
      """
    And the exit code is 4

  Scenario: --test selects named tests
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "cuke", "command": "echo running cuke tests" },
          { "name": "slow", "command": "echo running slow tests" }
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

  Scenario: commands.ci.test runs those tests by default
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "cuke", "command": "echo running cuke tests" }
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
      test all (unit)
      running unit tests
      """
    And it does not print
      """
      running cuke tests
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
