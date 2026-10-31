Feature: "trident fix-unsafe --scope" selects which files to fix

  Background:
    Given a Git repository
    And a committed file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """
    And a committed file ".ruff.toml" with content
      """
      [lint]
      extend-select = ["B006"]
      """
    And a committed file "on-main.py" with content
      """
      def add_to_list(item, items=[]):
          items.append(item)
          return items
      """
    And I ran "git checkout -b feature"
    And a committed file "committed-on-branch.py" with content
      """
      def add_to_list(item, items=[]):
          items.append(item)
          return items
      """
    And a file "untracked.py" with content
      """
      def add_to_list(item, items=[]):
          items.append(item)
          return items
      """

  Scenario: --scope=uncommitted fixes only uncommitted files
    When executing "trident fix-unsafe --scope=uncommitted --show=output"
    Then it prints to STDERR
      """
      1 Python
      running 1 tools
      """
    And it prints the lines
      """
      unsafe-fix Python (Ruff)
      Found 1 error (1 fixed, 0 remaining).
      """
    And the exit code is 0
    And file "untracked.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items
      """
    And file "committed-on-branch.py" is unchanged
    And file "on-main.py" is unchanged

  Scenario: --scope=branch fixes files changed on the current branch
    When executing "trident fix-unsafe --scope=branch --show=output"
    Then it prints to STDERR
      """
      2 Python
      running 1 tools
      """
    And it prints the lines
      """
      Found 2 errors (2 fixed, 0 remaining).
      """
    And the exit code is 0
    And file "committed-on-branch.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items
      """
    And file "untracked.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items
      """
    And file "on-main.py" is unchanged

  Scenario: --scope=all fixes all files
    When executing "trident fix-unsafe --scope=all --show=output"
    Then it prints to STDERR
      """
      3 Python, 1 other
      running 1 tools
      """
    And it prints the lines
      """
      Found 3 errors (3 fixed, 0 remaining).
      """
    And the exit code is 0
    And file "committed-on-branch.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items
      """
    And file "untracked.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items
      """
    And file "on-main.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items
      """
