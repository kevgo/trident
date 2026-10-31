Feature: pitstop Python

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """

  Scenario: unformatted Python
    Given a file "main.py" with content
      """
      print   ("Hello, world!")
      """
    When executing "trident pitstop --show=output"
    Then it prints
      """
      fix all (delete empty folders)
      fix Python (Ruff)
      All checks passed!
      format Python (Ruff)
      1 file reformatted
      lint Python (Ruff)
      All checks passed!
      """
    And the exit code is 0
    And file "main.py" now has content
      """
      print("Hello, world!")
      """

  Scenario: unformatted Python with lint errors
    Given a file "main.py" with content
      """
      def greet():
        a   =   1
      """
    When executing "trident pitstop --show=output"
    Then it prints
      """
      fix all (delete empty folders)
      fix Python (Ruff)
      F841 Local variable `a` is assigned to but never used
       --> main.py:2:3
        |
      1 | def greet():
      2 |   a   =   1
        |   ^
        |
      help: Remove assignment to unused variable `a`

      Found 1 error.
      No fixes available (1 hidden fix can be enabled with the `--unsafe-fixes` option).
      """
    And the exit code is 1
    And file "main.py" is unchanged
