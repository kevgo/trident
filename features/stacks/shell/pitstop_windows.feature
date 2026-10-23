@windows
Feature: pitstop Shell on Windows

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      shellcheck 0.11.0
      shfmt 3.14.1
      """

  Scenario: unformatted Shell
    Given a file "one.sh" with content
      """
      #!/usr/bin/env bash
      foo()   {
      echo   "one"
      }
      foo
      """
    When executing "trident pitstop --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And all files are unchanged

  Scenario: unformatted Shell with lint errors
    Given a file "one.sh" with content
      """
      #!/usr/bin/env bash
      foo()   {
      echo   "one"
      }
      foo
      """
    When executing "trident pitstop --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And all files are unchanged
