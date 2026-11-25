Feature: "trident lint" does not run "git diff --check" if not a Git repository

  Scenario: Git not installed
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a file "main.txt" with unix newlines
      """
      line one
      <<<<<<< HEAD:main.txt
      Hello world
      =======
      Goodbye
      >>>>>>> 77976da35a11db4580b80ae27e8d65caf5208086:main.txt
      """
    When executing "trident lint"
    Then it prints nothing to STDERR
    And it prints nothing to STDOUT
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
