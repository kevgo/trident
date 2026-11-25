Feature: precommit Python

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """

  Scenario: valid Python
    And a file "main.py" with content
      """
      print("Hello, world!")
      """
    And a file "other.py" with unix newlines
      """
      print("Hello, other!")
      """
    And I ran "git add main.py other.py"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.py" is unchanged
    And file "other.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """

  Scenario: unformatted Python
    Given a file "main.py" with content
      """
      print   ("Hello, world!")
      """
    And a file "other.py" with unix newlines
      """
      print   ("Hello, other!")
      """
    And I ran "git add main.py other.py"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.py" now has content
      """
      print("Hello, world!")
      """
    And file "other.py" now has content
      """
      print("Hello, other!")
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      git add -- main.py other.py
      """

  Scenario: invalid Python
    Given a file "main.py" with content
      """
      print("
      """
    And a file "other.py" with unix newlines
      """
      print("
      """
    And I ran "git add main.py other.py"
    When executing "trident precommit --show=failed"
    Then it prints
      """
      invalid-syntax: missing closing quote in string literal
       --> main.py:1:7
        |
      1 | print("
        |       ^
        |

      invalid-syntax: unexpected EOF while parsing
       --> main.py:2:1
        |
      1 | print("
        |        ^
        |

      invalid-syntax: missing closing quote in string literal
       --> other.py:1:7
        |
      1 | print("
        |       ^
        |

      invalid-syntax: unexpected EOF while parsing
       --> other.py:2:1
        |
      1 | print("
        |        ^
        |

      Found 4 errors.
      """
    And the exit code is 0
    And file "main.py" is unchanged
    And file "other.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """
