@online
Feature: lint SQL

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      uv 0.11.20
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "git_diff_check": {
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
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "one.sql" is unchanged
    And file "two.sql" is unchanged

  Scenario: unformatted SQL
    Given a file "one.sql" with content
      """
      SELECT            id, name FROM one
      """
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "one.sql" is unchanged

  Scenario: invalid SQL
    Given a file "one.sql" with content
      """
      SELECT FROM "
      """
    And a file "two.sql" with content
      """
      SELECT FROM "
      """
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "one.sql" is unchanged
    And file "two.sql" is unchanged

  Scenario Outline: unsupported SQL flavors
    Given a file "migration.<FILE EXTENSION>" with content
      """
      CREATE TABLE orders (id INT, total DECIMAL(10,2));
      """
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 JSON, 1 Markdown, 1 Python, 1 TOML, 5 other
      running 0 tools
      """
    And the exit code is 0
    And file "migration.<FILE EXTENSION>" is unchanged

    Examples:
      | FILE EXTENSION |
      | pgsql          |
      | tsql           |
