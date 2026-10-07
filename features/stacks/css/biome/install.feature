@online
Feature: install all CSS tools

  Scenario: not installed
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    Given a file "main.css" with content
      """
      .foo {
        color:    red;
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/biomejs/biome/releases/latest) ... ok
      """
    And it prints the block
      """
      fix CSS (Biome)
      """
    And the exit code is 0
    And file "main.css" now has content
      """
      .foo {
      \tcolor: red;
      }
      """
    And file "run-that-app" now has an additional line matching
      """
      biome \d+\.\d+\.\d+
      """
