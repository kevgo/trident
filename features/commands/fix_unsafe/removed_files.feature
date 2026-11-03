Feature: "trident fix-unsafe" operates only on changed files that still exist in the workspace

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
    And a committed file "unrelated.py" with content
      """
      def add_to_list(item, items=[]):
          items.append(item)
          return items
      """
    And a committed file "removed.py" with content
      """
      def add_to_list(item, items=[]):
          items.append(item)
          return items
      """

  Scenario: uncommitted deletion
    Given I delete file "removed.py"
    When executing "trident fix-unsafe --show=output"
    Then it prints to STDERR
      """
      1 Python
      running 0 tasks
      """
    And it does not print
      """
      unsafe-fix Python
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: uncommitted deletion next to a changed file
    Given a file "untracked.py" with content
      """
      def add_to_list(item, items=[]):
          items.append(item)
          return items
      """
    And I delete file "removed.py"
    When executing "trident fix-unsafe --show=output"
    Then it prints to STDERR
      """
      2 Python
      running 1 tasks
      """
    And it prints the lines
      """
      unsafe-fix Python (Ruff)
      Found 1 error (1 fixed, 0 remaining).
      """
    And the exit code is 0
    And file "unrelated.py" is unchanged
    And file "untracked.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items
      """

  Scenario: deletion committed on the feature branch
    Given I ran "git checkout -b feature"
    And I delete file "removed.py"
    And I ran "git add -A"
    And I ran "git commit -m delete"
    When executing "trident fix-unsafe --show=output"
    Then it prints to STDERR
      """
      1 Python
      running 0 tasks
      """
    And it does not print
      """
      unsafe-fix Python
      """
    And the exit code is 0
    And all files are unchanged
