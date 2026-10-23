Feature: fix Shell

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      shellcheck 0.11.0
      shfmt 3.14.1
      """

  Scenario: valid Shell
    Given a file "one.sh" with content
      """
      #!/usr/bin/env bash
      foo() {
      \techo "one"
      }
      foo
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Shell (shfmt)
      """
    And the exit code is 0
    And file "one.sh" is unchanged

  Scenario: unformatted Shell
    Given a file "one.sh" with content
      """
      #!/usr/bin/env bash
      if [ "$1" = "a" ]
      then
            echo   "a"
      fi
      """
    And a file "two.sh" with content
      """
      #!/usr/bin/env bash
      foo()   {
      echo   "two"
      }
      foo
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Shell (shfmt)
      """
    And the exit code is 0
    And file "one.sh" now has content
      """
      #!/usr/bin/env bash
      if [ "$1" = "a" ]; then
      \techo "a"
      fi
      """
    And file "two.sh" now has content
      """
      #!/usr/bin/env bash
      foo() {
      \techo "two"
      }
      foo
      """

  Scenario: invalid Shell
    Given a file "one.sh" with content
      """
      #!/usr/bin/env bash
      if [ "$1" = "a" ]
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Shell (shfmt)
      one.sh:2:1: `if <cond>` must be followed by `then`
      """
    And the exit code is 1
    And file "one.sh" is unchanged

  Scenario: Shell with lint errors
    Given a file "one.sh" with content
      """
      #!/usr/bin/env bash
      echo $1
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Shell (shfmt)
      """
    And the exit code is 0
    And file "one.sh" is unchanged
