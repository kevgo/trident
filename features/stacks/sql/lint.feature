Feature: lint SQL

  Background:
    Given a file "run-that-app" with unix newlines
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      sqlfmt 0.32.0
      uv 0.13.0
      """
    And a file "trident.json" with unix newlines
      """
      {
        "applications": {
          "biome": {
            "enabled": false
          },
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

  Scenario: valid SQL
    Given a file "one.sql" with unix newlines
      """
      select id, name from one
      """
    And a file "two.sql" with unix newlines
      """
      select id, name from two
      """
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "one.sql" is unchanged
    And file "two.sql" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted SQL
    Given a file "one.sql" with unix newlines
      """
      SELECT            id, name FROM one
      """
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "one.sql" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid SQL
    Given a file "one.sql" with unix newlines
      """
      SELECT FROM "
      """
    And a file "two.sql" with unix newlines
      """
      SELECT FROM "
      """
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "one.sql" is unchanged
    And file "two.sql" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario Outline: unsupported SQL flavors
    Given a file "migration.<FILE EXTENSION>" with unix newlines
      """
      CREATE TABLE orders (id INT, total DECIMAL(10,2));
      """
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 JSON, 2 other
      running 0 tasks
      """
    And the exit code is 0
    And file "migration.<FILE EXTENSION>" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

    Examples:
      | FILE EXTENSION |
      | pgsql          |
      | tsql           |
