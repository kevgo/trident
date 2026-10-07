@online
Feature: install sqlfmt
	# Trident doesn't auto-install SQLfmt.
  # SQLfmt needs to be installed by the user
  # by adding it to pyproject.toml.
  # We don't want to auto-install it
  # because it applies only to SQL files in dbt flavor.

  @this
  Scenario: not installed
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "git_diff_check": {
            "enabled": false
          },
          "prettier": {
            "enabled": false
          },
          "rumdl": {
            "enabled": false
          },
          "ruff": {
            "enabled": false
          },
          "taplo": {
            "enabled": false
          }
        }
      }
      """
    And a file "one.sql" with content
      """
      SELECT    id, name FROM one
      """
    When executing "trident fix --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/astral-sh/uv/releases/latest) ... ok
      """
    And it prints the lines
      """
      fix SQL (sqlfmt)
      xx
      """
    And the exit code is 0
    And file "one.sql" now has content
      """
      select id, name from one
      """
    And file "run-that-app" now has an additional line matching
      """
      delete-empty-folders \d+\.\d+\.\d+
      uv \d+\.\d+\.\d+
      """
