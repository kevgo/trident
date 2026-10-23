@unix
Feature: precommit Shell

  Background:
    Given a Git repository
    And a file "run-that-app" with content
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
    And I ran "git add one.sh"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "one.sh" is unchanged

  Scenario: unformatted Shell
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
      foo()   {
      echo   "two"
      }
      foo
      """
    And I ran "git add one.sh two.sh"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
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
    And I ran "git add one.sh"
    When executing "trident precommit --show=failed"
    Then it prints the lines
      """
      one.sh:2:1: `if <cond>` must be followed by `then`
      """
    And the exit code is 0
    And file "one.sh" is unchanged
