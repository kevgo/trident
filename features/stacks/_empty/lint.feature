Feature: checking a codebase without any code

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """

  Scenario: --show=output
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 other
      running 0 tasks
      """
    And it prints nothing to STDOUT
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: --show=names
    When executing "trident lint --show=names"
    Then it prints nothing to STDERR
    And it prints nothing to STDOUT
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: --show=failed
    When executing "trident lint --show=failed"
    Then it prints nothing to STDERR
    And it prints nothing to STDOUT
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
