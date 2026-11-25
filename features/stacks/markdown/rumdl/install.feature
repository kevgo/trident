@online
Feature: install Rumdl

  Scenario: not installed
    Given a file "run-that-app" with content
      """
      # more info at https://github.com/kevgo/run-that-app

      delete-empty-folders 0.0.2
      """
    And a file "main.md" with content
      """
      #     Hello
      """
    And a file "trident.json" with unix newlines
      """
      {
        "applications": {
          "biome": { "enabled": false }
        }
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/rvben/rumdl/releases/latest) ... ok
      """
    And it prints the lines
      """
      fix all (delete empty folders)
      fix Markdown (rumdl)
      main.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      main.md:1:2: [MD064] Multiple consecutive spaces (5) found [fixed]

      Fixed: 2/2 issues in 1 file (0ms)
      """
    And the exit code is 0
    And file "main.md" now has content
      """
      # Hello
      """
    And file "run-that-app" now has an additional line matching
      """
      rumdl \d+\.\d+\.\d+
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
