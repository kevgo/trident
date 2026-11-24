@unix
Feature: lint Shell on Unix

  Background:
    Given a file "run-that-app" with unix newlines
      """
      delete-empty-folders 0.0.2
      shellcheck 0.11.0
      shfmt 3.14.1
      """

  Scenario: valid Shell
    Given a file "one.sh" with unix newlines
      """
      #!/usr/bin/env bash
      echo "one"
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Shell (ShellCheck)
      """
    And the exit code is 0
    And file "one.sh" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Shell
    Given a file "one.sh" with unix newlines
      """
      #!/usr/bin/env bash
      foo()   {
      echo   "one"
      }
      foo
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Shell (ShellCheck)
      """
    And the exit code is 0
    And file "one.sh" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid Shell
    Given a file "one.sh" with unix newlines
      """
      #!/usr/bin/env bash
      echo $1
      """
    And a file "two.sh" with unix newlines
      """
      #!/usr/bin/env bash
      cd /tmp
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Shell (ShellCheck)
      In one.sh line 2:
      echo $1
           ^-- SC2086 (info): Double quote to prevent globbing and word splitting.
      """
    And it prints the lines
      """
      In two.sh line 2:
      cd /tmp
      ^-----^ SC2164 (warning): Use 'cd ... || exit' or 'cd ... || return' in case cd fails.
      """
    And the exit code is 1
    And file "one.sh" is unchanged
    And file "two.sh" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
