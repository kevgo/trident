Feature: "trident fix-unsafe" chooses the smallest "--scope" based on the workspace state

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

  Scenario: on main branch, no uncommitted files
    When executing "trident fix-unsafe --show=output"
    Then it prints to STDERR
      """
      1 Python, 1 other
      running 1 tools
      """
    And it prints the lines
      """
      Found 1 error (1 fixed, 0 remaining).
      """
    And the exit code is 0
    And file "on-main.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items
      """

  Scenario: on main branch, has uncommitted files
    And a file "untracked.py" with content
      """
      def add_to_list(item, items=[]):
          items.append(item)
          return items
      """
    When executing "trident fix-unsafe --show=output"
    Then it prints to STDERR
      """
      1 Python
      running 1 tools
      """
    And it prints the lines
      """
      unsafe-fix Python (ruff)
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
    And file "on-main.py" is unchanged

  Scenario: on feature branch, branch has no commits, no uncommitted files,
    Given I ran "git checkout -b feature"
    When executing "trident fix-unsafe --show=output"
    Then it prints to STDERR
      """
      1 Python, 1 other
      running 1 tools
      """
    And it prints the lines
      """
      Found 1 error (1 fixed, 0 remaining).
      """
    And the exit code is 0
    And file "on-main.py" now has content
      """
      def add_to_list(item, items=None):
          if items is None:
              items = []
          items.append(item)
          return items
      """

  Scenario: on feature branch, branch has commits, no uncommitted files,
    Given I ran "git checkout -b feature"
    And a committed file "committed-on-branch.py" with content
      """
      def add_to_list(item, items=[]):
          items.append(item)
          return items
      """
    When executing "trident fix-unsafe --show=output"
    Then it prints to STDERR
      """
      1 Python
      running 1 tools
      """
    And it prints the lines
      """
      Found 1 error (1 fixed, 0 remaining).
      """
    And it does not print
      """
      on-main.py
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
    And file "on-main.py" is unchanged

  Scenario: on feature branch, branch has commits, has uncommitted files
    Given I ran "git checkout -b feature"
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
    When executing "trident fix-unsafe --show=output"
    Then it prints to STDERR
      """
      1 Python
      running 1 tools
      """
    And it prints the lines
      """
      unsafe-fix Python (ruff)
      Found 1 error (1 fixed, 0 remaining).
      """
    And it does not print
      """
      committed-on-branch.py
      """
    And it does not print
      """
      on-main.py
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
