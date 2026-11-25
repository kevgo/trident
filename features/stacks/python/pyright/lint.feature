@online
Feature: lint Python with Pyright

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      pyright 1.1.414
      ruff 0.15.16
      """
    And a file "trident.json" with unix newlines
      """
      {
        "applications": {
          "git_diff_check": {
            "enabled": false
          },
          "rumdl": {
            "enabled": false
          },
          "taplo": {
            "enabled": false
          }
        }
      }
      """
    And a file "pyrightconfig.json" with unix newlines
      """
      {
        "typeCheckingMode": "strict"
      }
      """

  Scenario: valid Python with Pyright
    Given a file "main.py" with content
      """
      print("Hello, world!")
      """
    And a file "other.py" with content
      """
      print("Hello, other!")
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Python (Ruff)
      All checks passed!
      """
    And it prints the block
      """
      type-check Python (Pyright)
      0 errors, 0 warnings, 0 informations
      """
    And the exit code is 0
    And file "main.py" is unchanged
    And file "other.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Python
    Given a file "main.py" with content
      """
      print   ("Hello, world!")
      """
    And a file "other.py" with content
      """
      print   ("Hello, other!")
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Python (Ruff)
      All checks passed!
      """
    And it prints the block
      """
      type-check Python (Pyright)
      0 errors, 0 warnings, 0 informations
      """
    And the exit code is 0
    And file "main.py" is unchanged
    And file "other.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: typecheck errors
    Given a file "main.py" with content
      """
      a: int = "text"
      print(a)
      """
    And a file "other.py" with content
      """
      b: int = "text"
      print(b)
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Python (Ruff)
      All checks passed!
      lint JSON (Biome)
      Checked 2 files in 1000µs. No fixes applied.
      type-check Python (Pyright)
      /tmp/.tmprw5mGf/16445664057987328386/main.py
        /tmp/.tmprw5mGf/16445664057987328386/main.py:1:10 - error: Type "Literal['text']" is not assignable to declared type "int"
          "Literal['text']" is not assignable to "int" (reportAssignmentType)
      /tmp/.tmprw5mGf/16445664057987328386/other.py
        /tmp/.tmprw5mGf/16445664057987328386/other.py:1:10 - error: Type "Literal['text']" is not assignable to declared type "int"
          "Literal['text']" is not assignable to "int" (reportAssignmentType)
      2 errors, 0 warnings, 0 informations
      """
    And the exit code is 1
    And file "main.py" is unchanged
    And file "other.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid Python
    Given a file "main.py" with content
      """
      print(123
      """
    And a file "other.py" with content
      """
      print("
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Python (Ruff)
      invalid-syntax: unexpected EOF while parsing
      """
    And it prints the lines
      """
      lint Python (Ruff)
      invalid-syntax: unexpected EOF while parsing
       --> main.py:2:1
        |
      1 | print(123
        |          ^
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

      Found 3 errors.
      lint JSON (Biome)
      Checked 2 files in 1666µs. No fixes applied.
      type-check Python (Pyright)
      /tmp/.tmpjKZCNh/13096620741874266971/main.py
        /tmp/.tmpjKZCNh/13096620741874266971/main.py:1:6 - error: "(" was not closed
        /tmp/.tmpjKZCNh/13096620741874266971/main.py:2:1 - error: Statements must be separated by newlines or semicolons
      /tmp/.tmpjKZCNh/13096620741874266971/other.py
        /tmp/.tmpjKZCNh/13096620741874266971/other.py:1:6 - error: "(" was not closed
        /tmp/.tmpjKZCNh/13096620741874266971/other.py:1:7 - error: String literal is unterminated
        /tmp/.tmpjKZCNh/13096620741874266971/other.py:2:1 - error: Statements must be separated by newlines or semicolons
      5 errors, 0 warnings, 0 informations
      """
    And the exit code is 1
    And file "main.py" is unchanged
    And file "other.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
