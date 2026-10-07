@online
Feature: pitstop SQL

  Background:
    Given a file "run-that-app" with content
      """
      uv 0.11.20
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "delete_empty_folders": {
            "enabled": false
          },
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
    And I ran "tools/rta uv init"
    And I ran "tools/rta uv add shandy-sqlfmt[jinjafmt]"

  Scenario: unformatted SQL
    Given a file "one.sql" with content
      """
      SELECT            id, name FROM one
      """
    And a file "two.sql" with content
      """
      SELECT            id, name FROM two
      """
    When executing "trident pitstop --show=output"
    Then it prints the block
      """
      fix SQL (sqlfmt)
      2 files formatted.
      0 files left unchanged.
      one.sql formatted.
      two.sql formatted.
      """
    And the exit code is 0
    And file "one.sql" now has content
      """
      select id, name from one
      """
    And file "two.sql" now has content
      """
      select id, name from two
      """

  Scenario: unformatted SQL with lint errors
    Given a file "one.sql" with content
      """
      SELECT INTO
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      delete empty folders
      fix SQL (sqlfmt)
      1 file left unchanged.
      """
    And the exit code is 0
    And file "one.sql" is unchanged
