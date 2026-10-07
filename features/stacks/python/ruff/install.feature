@online
Feature: install all Python tools

  Scenario: not installed
    Given a file "run-that-app" with content
      """
      node 26.4.0
      prettier 3.7.0
      """
    And a file "main.py" with content
      """
      print    ("Hello, world!")
      """
    And a file "pyrightconfig.json" with content
      """
      {
        "typeCheckingMode": "strict"
      }
      """
    When executing "trident lint --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/astral-sh/uv/releases/latest) ... ok
      Talking to GitHub API (https://api.github.com/repos/astral-sh/ruff/releases/latest) ... ok
      """
    And it prints the block
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
    And file "run-that-app" now has an additional line matching
      """
      ruff \d+\.\d+\.\d+
      uv \d+\.\d+\.\d+
      """
