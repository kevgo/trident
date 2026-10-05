Feature: lint Python with Pyright

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      uv 0.11.20
      """
    And a file "pyrightconfig.json" with content
      """
      {
        "typeCheckingMode": "strict"
      }
      """
    And I ran "tools/rta uv tool install pyright"

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
      lint Python (ruff)
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
      lint Python (ruff)
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
      type-check Python (Pyright)
      2 errors, 0 warnings, 0 informations
      """
    And the exit code is 1
    And file "main.py" is unchanged
    And file "other.py" is unchanged

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
      lint Python (ruff)
      invalid-syntax: unexpected EOF while parsing
      """
    And it prints the lines
      """
      type-check Python (Pyright)
      5 errors, 0 warnings, 0 informations
      """
    And the exit code is 1
    And file "main.py" is unchanged
    And file "other.py" is unchanged
