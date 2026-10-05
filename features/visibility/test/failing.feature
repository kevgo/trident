Feature: test command with a failing test

  Background:
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit tests", "command": "echo unit" },
          { "name": "E2E tests", "command": "tests/fail.sh" }
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
      unit tests
      unit
      """
    And it prints the block
      """
      E2E tests
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
      unit tests
      (sh|bash.exe) -c 'echo unit'
      unit
      """
    And it prints the lines matching
      """
      E2E tests
      (sh|bash.exe) -c tests/fail.sh
      failed
      """
    And the exit code is 4

  Scenario: --show=names
    When executing "trident test --show=names"
    Then it prints nothing to STDERR
    And it prints the lines
      """
      unit tests
      """
    And it prints the block
      """
      E2E tests
      failed
      """
    And the exit code is 4

  Scenario: --show=failed
    When executing "trident test --show=failed"
    Then it prints nothing to STDERR
    And it does not print
      """
      unit tests
      """
    And it prints the block
      """
      failed
      """
    And the exit code is 4
