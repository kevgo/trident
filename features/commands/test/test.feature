Feature: custom tests

  Scenario: tests pass
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "command": "tests/one.sh" },
          {
            "name": "list all files",
            "command": "find . | sort | xargs echo"
          }
        ]
      }
      """
    And an executable file "tests/one.sh" with content
      """
      #!/usr/bin/env bash
      echo "custom test is running"
      """
    When executing "trident test --show=output"
    Then it prints the block
      """
      test all (tests/one.sh)
      custom test is running
      """
    And it prints the block
      """
      test all (list all files)
      . ./tests ./tests/one.sh ./trident.json
      """
    And the exit code is 0

  Scenario: test fails
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "command": "tests/fail.sh" }
        ]
      }
      """
    And an executable file "tests/fail.sh" with content
      """
      #!/usr/bin/env bash
      echo "custom test failed"
      exit 4
      """
    When executing "trident test --show=output"
    Then it prints the block
      """
      test all (tests/fail.sh)
      custom test failed
      """
    And the exit code is 4

  Scenario: unnamed test uses the command as the name
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "command": "echo unnamed" }
        ]
      }
      """
    When executing "trident test --show=output"
    Then it prints the block
      """
      test all (echo unnamed)
      unnamed
      """
    And the exit code is 0

  Scenario: missing tests succeeds
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 0 tools
      """
    And it prints nothing to STDOUT
    And the exit code is 0

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
    When executing "trident test --test=unit+cuke --show=output"
    Then it prints to STDERR
      """
      running 2 tools
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

  Scenario: commands.test.test runs those tests by default
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "cuke", "command": "echo running cuke tests" }
        ],
        "commands": {
          "test": {
            "test": ["unit"]
          }
        }
      }
      """
    When executing "trident test --show=output"
    Then it prints to STDERR
      """
      running 1 tools
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
    When executing "trident test --test=unit+missing"
    Then it prints
      """
      unknown test: missing
      available tests: unit, cuke
      """
    And the exit code is 1
