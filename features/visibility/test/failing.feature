Feature: test command with a failing test

  Background:
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "E2E", "command": "tests/fail.sh" }
        ]
      }
      """
    And an executable file "tests/fail.sh" with content
      """
      #!/usr/bin/env bash
      echo failed
      exit 4
      """

  Scenario: --show=output
    When executing "trident test --show=output"
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
      test all (E2E)
      failed
      """
    And the exit code is 4

  Scenario: --show=verbose
    When executing "trident test --show=verbose"
    Then it prints to STDERR
      """
      running 2 tools
      """
    And it prints the lines matching
      """
      test all \(unit\)
      (sh|bash.exe) -c 'echo running unit tests'
      running unit tests
      """
    And it prints the lines matching
      """
      test all \(E2E\)
      (sh|bash.exe) -c tests/fail.sh
      failed
      """
    And the exit code is 4

  Scenario: --show=names
    When executing "trident test --show=names"
    Then it prints nothing to STDERR
    And it prints the lines
      """
      test all (unit)
      """
    And it prints the block
      """
      test all (E2E)
      failed
      """
    And the exit code is 4

  Scenario: --show=failed
    When executing "trident test --show=failed"
    Then it prints nothing to STDERR
    And it does not print
      """
      test all (unit)
      """
    And it prints the block
      """
      failed
      """
    And the exit code is 4
