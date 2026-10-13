Feature: fix SQL

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      sqlfmt 0.32.0
      uv 0.13.0
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "delete-empty-folders": {
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

  Scenario: valid SQL
    Given a file "one.sql" with content
      """
      select id, name from one
      """
    And a file "two.sql" with content
      """
      select id, name from two
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix SQL (sqlfmt)
      2 files left unchanged.
      """
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
    When executing "trident fix --show=output"
    Then it prints the lines
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

  Scenario: invalid SQL
    Given a file "one.sql" with content
      """
      SELECT FROM "
      """
    And a file "two.sql" with content
      """
      SELECT FROM "
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix SQL (sqlfmt)
      2 files had errors while formatting.
      0 files left unchanged.
      one.sql
          sqlfmt encountered an error: Could not parse SQL at position 11: '"'
      two.sql
          sqlfmt encountered an error: Could not parse SQL at position 11: '"'
      """
    And the exit code is 2
    And file "one.sql" is unchanged
    And file "two.sql" is unchanged

  Scenario Outline: unsupported SQL flavors
    Given a file "migration.<FILE EXTENSION>" with content
      """
      CREATE TABLE orders (id INT, total DECIMAL(10,2));
      """
    When executing "trident fix --show=output"
    Then it prints to STDERR
      """
      1 JSON, 2 other
      running 0 tools
      """
    And the exit code is 0
    And file "migration.<FILE EXTENSION>" is unchanged

    Examples:
      | FILE EXTENSION |
      | pgsql          |
      | tsql           |
