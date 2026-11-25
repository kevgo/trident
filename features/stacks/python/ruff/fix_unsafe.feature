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
    And a file "ruff.toml" with unix newlines
      """
      [lint]
      extend-select = ["B006"]
      """

  Scenario: fix-unsafe
    When executing "trident fix-unsafe --show=output"
    Then it prints the lines
      """
      unsafe-fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpH1jDPL/502752668947887438/ruff.toml"] cwd="/tmp/.tmpH1jDPL/502752668947887438"
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
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: fix
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpBFkKE0/4229047332084141781/ruff.toml"] cwd="/tmp/.tmpBFkKE0/4229047332084141781"
      fix Python (Ruff)
      B006 Do not use mutable data structures for argument defaults
       --> main.py:1:29
        |
      1 | def add_to_list(item, items=[]):
        |                             ^^
      2 |     items.append(item)
      3 |     return items
        |
      help: Replace with `None`; initialize within function

      Found 1 error.
      No fixes available (1 hidden fix can be enabled with the `--unsafe-fixes` option).
      """
    And the exit code is 1
    And file "main.py" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
