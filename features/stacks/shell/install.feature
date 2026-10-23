@online
Feature: install all Shell tools

  Scenario: not installed
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a file "one.sh" with content
      """
      #!/usr/bin/env bash
      foo()   {
      echo   "one"
      }
      foo
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/mvdan/sh/releases/latest) ... ok
      """
    And it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/koalaman/shellcheck/releases/latest) ... ok
      """
    And it prints the lines
      """
      fix Shell (shfmt)
      lint Shell (Shellcheck)
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
    And file "run-that-app" now has an additional line matching
      """
      shellcheck \d+\.\d+\.\d+
      shfmt \d+\.\d+\.\d+
      """
