@online
Feature: install Text-Runner

  Scenario: not installed
    Given a file "main.md" with content
      """
      # Hello
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false },
          "rumdl": { "enabled": false },
          "tikibase": { "enabled": false }
        }
      }
      """
    And a file "text-runner.jsonc" with content
      """
      {
        "format": "detailed",
        "systemTmp": true,
        "online": false
      }
      """
    When executing "trident lint --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/nodejs/node/releases/latest) ... ok
      Talking to GitHub API (https://api.github.com/repos/kevgo/text-runner/releases/latest) ... ok
      """
    And it prints the lines
      """
      test Markdown (Text-Runner)
      """
    And the exit code is 0
    And file "main.md" now has content
      """
      # Hello
      """
    And file "run-that-app" now has an additional line matching
      """
      node \d+\.\d+\.\d+
      text-runner \d+\.\d+\.\d+
      """
