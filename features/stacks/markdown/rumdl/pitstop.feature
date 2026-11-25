Feature: pitstop Markdown

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """

  Scenario: unformatted Markdown
    Given a file "main.md" with unix newlines
      """
      #     Hello
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix Markdown (rumdl)
      main.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      """
    And the exit code is 0
    And file "main.md" now has content
      """
      # Hello
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: file with spaces in the name
    Given a file "my file.md" with unix newlines
      """
      #     Hello
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix Markdown (rumdl)
      my file.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      """
    And the exit code is 0
    And file "my file.md" now has content
      """
      # Hello
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Markdown with lint errors
    Given a file "main.md" with unix newlines
      """
      one



      two
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix Markdown (rumdl)
      main.md:1:1: [MD041] First line in file should be a level 1 heading
      lint Markdown (rumdl)
      main.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And the exit code is 1
    And file "main.md" now has content
      """
      one

      two
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
