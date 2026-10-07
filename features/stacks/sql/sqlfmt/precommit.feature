Feature: precommit SQL

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
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

  Scenario: valid SQL
    Given a file "one.sql" with content
      """
      select id, name from one
      """
    And a file "two.sql" with content
      """
      select id, name from two
      """
    And I ran "git add one.sql two.sql"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "one.sql" is unchanged
    And file "two.sql" is unchanged

  Scenario: unformatted SQL
    Given a file "one.sql" with content
      """
      SELECT            id, name FROM one
      """
    And a file "two.sql" with content
      """
      SELECT            id, name FROM two
      """
    And I ran "git add one.sql two.sql"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "one.sql" now has content
      """
      select id, name from one
      """
    And file "two.sql" now has content
      """
      select id, name from two
      """

  Scenario: invalid SQL
    Given a file "one.sql" with content
      """
      SELECT FROM "
      """
    And a file "two.sql" with content
      """
      SELECT FROM "
      """
    And I ran "git add one.sql two.sql"
    When executing "trident precommit --show=failed"
    Then it prints
      """
      2 files had errors while formatting.
      0 files left unchanged.
      one.sql
          sqlfmt encountered an error: Could not parse SQL at position 11: '"'
      two.sql
          sqlfmt encountered an error: Could not parse SQL at position 11: '"'
      """
    And the exit code is 0
    And file "one.sql" is unchanged
    And file "two.sql" is unchanged

  Scenario Outline: unsupported SQL flavors
    Given a file "migration.<FILE EXTENSION>" with content
      """
      CREATE TABLE orders (id INT, total DECIMAL(10,2));
      """
    And I ran "git add migration.<FILE EXTENSION>"
    When executing "trident precommit --show=output"
    Then it prints to STDERR
      """
      1 other
      running 1 tools
      """
    And the exit code is 0
    And file "migration.<FILE EXTENSION>" is unchanged

    Examples:
      | FILE EXTENSION |
      | pgsql          |
      | tsql           |
