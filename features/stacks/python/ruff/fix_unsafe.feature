Feature: unsafe-fix Python

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      taplo 0.10.0
      """
    And a file "main.py" with content
      """
      def add_to_list(item, items=[]):
          items.append(item)
          return items


      print(add_to_list("a"))  # ["a"]
      print(add_to_list("b"))  # ["a", "b"]
      """
    And a file "ruff.toml" with content
      """
      [lint]
      extend-select = ["B006"]
      """

  Scenario: fix-unsafe
    When executing "trident fix-unsafe --show=output"
    Then it prints the lines
      """
      unsafe-fix Python (Ruff)
      Found 1 error (1 fixed, 0 remaining).
      format Python (Ruff)
      1 file left unchanged
      """
    And the exit code is 0
    And file "main.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items


      print(add_to_list("a"))  # ["a"]
      print(add_to_list("b"))  # ["a", "b"]
      """

  Scenario: fix
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      delete empty folders
      fix Python (Ruff)
      B006 Do not use mutable data structures for argument defaults
      No fixes available (1 hidden fix can be enabled with the `--unsafe-fixes` option).
      """
    And the exit code is 1
    And file "main.py" is unchanged
