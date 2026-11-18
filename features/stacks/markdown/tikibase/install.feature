@online
Feature: install Tikibase

  Scenario: not installed
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false },
          "rumdl": { "enabled": false }
        }
      }
      """
    And a file "tikibase.json" with content
      """
      {
        "ignore": [
          "run-that-app",
          "trident.json"
        ],
        "bidiLinks": false
      }
      """
    And a file "one.md" with content
      """
      # One
      [two](two.md)
      """
    And a file "two.md" with content
      """
      # Two
      [one](one.md)
      """
    When executing "trident lint --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/kevgo/tikibase/releases/latest) ... ok
      """
    And it prints the lines
      """
      lint Markdown (tikibase)
      """
    And the exit code is 0
    And file "run-that-app" now has an additional line matching
      """
      tikibase \d+\.\d+\.\d+
      """
