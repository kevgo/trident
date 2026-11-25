Feature: config file format

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """

  Scenario: comments and trailing commas are accepted
    Given a file "trident.json" with unix newlines
      """
      {
        // a comment
        "applications": {
          "biome": { "enabled": false }
        },
        "global-lints": [
          {
            "name": "hello",
            "command": "echo hello",
          },
        ],
      }
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint all (hello)
      hello
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: trident.jsonc is used when trident.json is absent
    Given a file "trident.jsonc" with unix newlines
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "global-lints": [
          { "name": "from jsonc", "command": "echo from jsonc" }
        ]
      }
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint all (from jsonc)
      from jsonc
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: trident.json takes precedence over trident.jsonc
    Given a file "trident.json" with unix newlines
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "global-lints": [
          { "name": "from json", "command": "echo from json" }
        ]
      }
      """
    And a file "trident.jsonc" with unix newlines
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "global-lints": [
          { "name": "from jsonc", "command": "echo from jsonc" }
        ]
      }
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint all (from json)
      from json
      """
    And it does not print
      """
      from jsonc
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unknown keys are rejected
    Given a file "trident.json" with unix newlines
      """
      {
        "unknown-key": true
      }
      """
    When executing "trident lint"
    Then it prints the block
      """
      config file (trident.json): unknown field `unknown-key`
      """
    And the exit code is 1
    And it runs no Git commands
