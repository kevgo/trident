Feature: unsafe-fix CSS

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """
    And a file "main.css" with unix newlines
      """
      .style {
      \tcolor: red !important;
      }
      """
    And a file "biome.json" with unix newlines
      """
      {
        "linter": {
          "rules": {
            "complexity": {
              "noImportantStyles": "error"
            }
          }
        }
      }
      """

  Scenario: fix-unsafe
    When executing "trident fix-unsafe --show=output"
    Then it prints the lines
      """
      unsafe-fix JSON (Biome)
      Checked 1 file in 845µs. No fixes applied.
      unsafe-fix CSS (Biome)
      Checked 1 file in 964µs. Fixed 1 file.
      """
    And the exit code is 0
    And file "main.css" now has content
      """
      .style {
      \tcolor: red;
      }
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: fix
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix JSON (Biome)
      Formatted 1 file in 285µs. Fixed 1 file.
      fix CSS (Biome)
      Formatted 1 file in 317µs. No fixes applied.
      """
    And the exit code is 0
    And file "main.css" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
