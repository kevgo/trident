Feature: formatting a codebase without any code

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """

  Scenario: --show=output
    When executing "trident fix --show=output"
    Then it prints
      """
      fix all (delete empty folders)
      """
    And it prints to STDERR
      """
      1 other
      running 1 tools
      """
    And the exit code is 0

  Scenario: --show=names
    When executing "trident fix --show=names"
    Then it prints
      """
      fix all (delete empty folders)
      """
    And the exit code is 0

  Scenario: --show=failed
    When executing "trident fix --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
