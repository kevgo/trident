Feature: "trident postedit" lints all files outside a Git repository

  Scenario: not a Git repository
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """
    And a file "main.md" with content
      """
      text
      """
    When executing "trident postedit --show=output"
    Then it prints the lines to STDERR
      """
      1 Markdown, 1 other
      running 1 tasks
      """
    And it prints the lines
      """
      lint Markdown (rumdl)
      main.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And the exit code is 1
    And file "main.md" is unchanged
