Feature: config file format

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """

  Scenario: comments and trailing commas are accepted
    Given a file "trident.json" with content
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
      hello
      hello
      """
    And the exit code is 0

  Scenario: trident.jsonc is used when trident.json is absent
    Given a file "trident.jsonc" with content
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
      from jsonc
      from jsonc
      """
    And the exit code is 0

  Scenario: trident.json takes precedence over trident.jsonc
    Given a file "trident.json" with content
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
    And a file "trident.jsonc" with content
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
      from json
      from json
      """
    And it does not print
      """
      from jsonc
      """
    And the exit code is 0

  Scenario: unknown keys are rejected
    Given a file "trident.json" with content
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
