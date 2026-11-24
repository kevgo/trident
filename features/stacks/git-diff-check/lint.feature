Feature: "trident lint" runs "git diff --check"

  Background:
    Given a Git repository
    And a file "run-that-app" with unix newlines
      """
      delete-empty-folders 0.0.2
      """
    And a committed file "main.txt" with content
      """
      line one
      """

  Scenario: clean repository
    When executing "trident lint --show=failed"
    Then it prints nothing to STDERR
    And it prints nothing to STDOUT
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --untracked-files=all
      git status --porcelain=v1 -z --untracked-files=all
      git diff HEAD --check
      """

  Scenario: whitespace error in a changed file
    Given I change file "main.txt" to
      """
      line one
       \tindented
      """
    When executing "trident lint"
    Then it prints the block
      """
      main.txt:2: space before tab in indent.
      """
    And the exit code is 2
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --untracked-files=all
      git status --porcelain=v1 -z --untracked-files=all
      git diff HEAD --check
      """

  Scenario: conflict markers in a changed file
    Given I change file "main.txt" to
      """
      line one
      <<<<<<< HEAD:main.txt
      Hello world
      =======
      Goodbye
      >>>>>>> 77976da35a11db4580b80ae27e8d65caf5208086:main.txt
      """
    When executing "trident lint"
    Then it prints the block
      """
      main.txt:2: leftover conflict marker
      main.txt:4: leftover conflict marker
      main.txt:6: leftover conflict marker
      """
    And the exit code is 2
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --untracked-files=all
      git status --porcelain=v1 -z --untracked-files=all
      git diff HEAD --check
      """
