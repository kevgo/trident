Feature: fix Markdown

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """

  Scenario: valid Markdown
    Given a file "main.md" with content
      """
      # Hello
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix Markdown (rumdl)

      Success: No issues found in 1 file (2ms)
      """
    And the exit code is 0
    And file "main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Markdown
    Given a file "main.md" with content
      """
      #     Hello
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix Markdown (rumdl)
      main.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      main.md:1:2: [MD064] Multiple consecutive spaces (5) found [fixed]

      Fixed: Fixed 2/2 issues in 1 file (5ms)
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

  Scenario: invalid Markdown
    Given a file "main.md" with content
      """
      # hello

      [e]()
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix Markdown (rumdl)
      main.md:3:1: [MD042] Empty link found: [e]()

      Issues: Found 1 issues in 1 file (4ms)
      """
    And the exit code is 0
    And file "main.md" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: file with spaces in the name
    Given a file "my file.md" with content
      """
      #     Hello
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix Markdown (rumdl)
      my file.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      my file.md:1:2: [MD064] Multiple consecutive spaces (5) found [fixed]

      Fixed: Fixed 2/2 issues in 1 file (3ms)
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
