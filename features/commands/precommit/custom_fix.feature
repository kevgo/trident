Feature: stack-scoped custom fixes

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      rumdl 0.2.14
      """
    And a file "trident.json" with content
      """
      {
        "stacks": {
          "python": {
            "fix": {
              "replace": [
                { "name": "my python fix", "command": "fixes/python.sh" }
              ]
            }
          }
        }
      }
      """
    And an executable file "fixes/python.sh" with content
      """
      #!/bin/sh
      echo "PYTHON FIX RAN"
      """
    And a file "main.py" with content
      """
      print("hello")
      """
    And a file "one.md" with unix newlines
      """
      # One
      """
    And I ran "git add -A"
    And I ran "git commit -m original"

  Scenario: a file of the stack is staged
    Given I change file "main.py" to
      """
      print("world")
      """
    And I ran "git add main.py"
    When executing "trident precommit --show=output"
    Then it prints the block
      """
      fix Python (my python fix)
      PYTHON FIX RAN
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """

  Scenario: no file of the stack is staged
    And I change file "one.md" to
      """
      # New one
      """
    And I ran "git add one.md"
    When executing "trident precommit --show=output"
    Then it does not print
      """
      my python fix
      """
    And it does not print
      """
      PYTHON FIX RAN
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """
