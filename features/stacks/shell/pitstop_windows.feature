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
    Then it prints
      """
      fix all (delete empty folders)
      fix Shell (shfmt)
      """
    And the exit code is 0
    And file "one.sh" now has content
      """
      #!/usr/bin/env bash
      foo() {
      \techo "one"
      }
      foo
      """

  Scenario: unformatted Shell with lint errors
    Given a file "one.sh" with content
      """
      #!/usr/bin/env bash
      foo()   {
      echo   "one"
      }
      foo
      """
    And a file "two.sh" with content
      """
      #!/usr/bin/env bash
      cd   /tmp
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix Shell (shfmt)
      """
    And the exit code is 0
    And file "one.sh" now has content
      """
      #!/usr/bin/env bash
      foo() {
      \techo "one"
      }
      foo
      """
    And file "two.sh" now has content
      """
      #!/usr/bin/env bash
      cd /tmp
      """
