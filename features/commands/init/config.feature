Feature: "trident init:config" writes the default config file

  Scenario: write trident.jsonc into an empty project
    When executing "trident init:config"
    Then it prints nothing to STDERR
    And it prints
      """
      created trident.jsonc
      """
    And the exit code is 0
    And file "trident.jsonc" now has content
      """
      {
        // link to the JSON schema for this file,
        // for auto-complete in VSCode and compatible editors
        "$schema": "https://github.com/kevgo/trident/raw/refs/heads/main/docs/schema.json",

        // These files are invisible to Trident.
        "ignore-files": ["node_modules/", "**/*.min.css"],

        // These tools always run.
        "global-lints": [],
        "global-fixes": [],

        // Define the functional tests.
        "tests": [],

        // configure the supported software stacks
        "stacks": {},

        // configure the built-in tools
        //
        // Only applications that can receive file paths as CLI arguments have an "ignore-files" key.
        "applications": {}
      }
      """
    And it runs no Git commands

  Scenario Outline: existing config file
    Given a file "<FILENAME>" with content
      """
      existing
      """
    When executing "trident init:config"
    Then it prints nothing to STDERR
    And it prints
      """
      config file <FILENAME> already exists
      """
    And the exit code is 1
    And file "<FILENAME>" is unchanged
    And it runs no Git commands

    Examples:
      | FILENAME      |
      | trident.json  |
      | trident.jsonc |

  Scenario Outline: force overwrites an existing config file
    Given a file "<FILENAME>" with content
      """
      existing
      """
    When executing "trident init:config <FLAG>"
    Then it prints nothing to STDERR
    And it prints
      """
      created <FILENAME>
      """
    And the exit code is 0
    And file "<FILENAME>" now has content
      """
      {
        // link to the JSON schema for this file,
        // for auto-complete in VSCode and compatible editors
        "$schema": "https://github.com/kevgo/trident/raw/refs/heads/main/docs/schema.json",

        // These files are invisible to Trident.
        "ignore-files": ["node_modules/", "**/*.min.css"],

        // These tools always run.
        "global-lints": [],
        "global-fixes": [],

        // Define the functional tests.
        "tests": [],

        // configure the supported software stacks
        "stacks": {},

        // configure the built-in tools
        //
        // Only applications that can receive file paths as CLI arguments have an "ignore-files" key.
        "applications": {}
      }
      """
    And it runs no Git commands

    Examples:
      | FILENAME      | FLAG    |
      | trident.json  | --force |
      | trident.json  | -f      |
      | trident.jsonc | --force |
      | trident.jsonc | -f      |
