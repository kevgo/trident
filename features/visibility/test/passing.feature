Feature: test command with passing tests

  Background:
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "E2E", "command": "echo running E2E tests" }
        ]
      }
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
      running E2E tests
      """
    And the exit code is 0

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
      (sh|bash.exe) -c 'echo running E2E tests'
      running E2E tests
      """
    And the exit code is 0

  Scenario: --show=names
    When executing "trident test --show=names"
    Then it prints only these lines in any order
      """
      test all (unit)
      test all (E2E)
      """
    And it prints nothing to STDERR
    And the exit code is 0

  Scenario: --show=failed
    When executing "trident test --show=failed"
    Then it prints nothing to STDOUT
    And it prints nothing to STDERR
    And the exit code is 0
