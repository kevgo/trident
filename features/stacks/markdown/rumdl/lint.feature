Feature: lint Markdown

  Background:
    Given a file "run-that-app" with unix newlines
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """

  Scenario: valid Markdown
    Given a file "main.md" with unix newlines
      """
      # Hello
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Markdown (rumdl)
      """
    And the exit code is 0
    And file "main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Markdown
    Given a file "main.md" with unix newlines
      """
      #     Hello
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Markdown (rumdl)
      main.md:1:2: [MD019] Multiple spaces (5) after # in heading [*]
      """
    And the exit code is 1
    And file "main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid Markdown
    Given a file "main.md" with unix newlines
      """
      text
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Markdown (rumdl)
      main.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And the exit code is 1
    And file "main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: file with spaces in the name
    Given a file "my file.md" with unix newlines
      """
      text
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Markdown (rumdl)
      my file.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And the exit code is 1
    And file "my file.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
