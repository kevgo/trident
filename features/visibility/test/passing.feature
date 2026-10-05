Feature: test command with passing tests

  Background:
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit tests", "command": "echo unit" },
          { "name": "E2E tests", "command": "echo e2e" }
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
      unit tests
      unit
      """
    And it prints the block
      """
      E2E tests
      e2e
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
      unit tests
      (sh|bash.exe) -c 'echo unit'
      unit
      """
    And it prints the lines matching
      """
      E2E tests
      (sh|bash.exe) -c 'echo e2e'
      e2e
      """
    And the exit code is 0

  Scenario: --show=names
    When executing "trident test --show=names"
    Then it prints only these lines in any order
      """
      unit tests
      E2E tests
      """
    And it prints nothing to STDERR
    And the exit code is 0

  Scenario: --show=failed
    When executing "trident test --show=failed"
    Then it prints nothing to STDOUT
    And it prints nothing to STDERR
    And the exit code is 0
