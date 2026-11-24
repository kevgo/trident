Feature: lint Python

  Background:
    Given a file "run-that-app" with unix newlines
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """

  Scenario: valid Python
    Given a file "main.py" with unix newlines
      """
      print("Hello, world!")
      """
    And a file "other.py" with unix newlines
      """
      print("Hello, other!")
      """
    When executing "trident lint --show=output"
    Then it prints
      """
      lint Python (Ruff)
      All checks passed!
      """
    And the exit code is 0
    And file "main.py" is unchanged
    And file "other.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Python
    Given a file "main.py" with unix newlines
      """
      print   ("Hello, world!")
      """
    And a file "other.py" with unix newlines
      """
      print   ("Hello, other!")
      """
    When executing "trident lint --show=output"
    Then the exit code is 0
    And file "main.py" is unchanged
    And file "other.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid Python
    Given a file "main.py" with unix newlines
      """
      print("
      """
    And a file "other.py" with unix newlines
      """
      print("
      """
    When executing "trident lint --show=output"
    Then the exit code is 1
    And file "main.py" is unchanged
    And file "other.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
