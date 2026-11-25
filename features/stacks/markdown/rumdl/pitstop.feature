Feature: pitstop Markdown

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """

  Scenario: unformatted Markdown
    Given a file "main.md" with content
      """
      #     Hello
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix Markdown (rumdl)
      main.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      main.md:1:2: [MD064] Multiple consecutive spaces (5) found [fixed]

      Fixed: Fixed 2/2 issues in 1 file (3ms)
      lint Markdown (rumdl)

      Success: No issues found in 1 file (2ms)
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
    Given a file "my file.md" with content
      """
      #     Hello
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix Markdown (rumdl)
      my file.md:1:2: [MD019] Multiple spaces (5) after # in heading [fixed]
      my file.md:1:2: [MD064] Multiple consecutive spaces (5) found [fixed]

      Fixed: Fixed 2/2 issues in 1 file (4ms)
      lint Markdown (rumdl)

      Success: No issues found in 1 file (3ms)
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
    Given a file "main.md" with content
      """
      one



      two
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix Markdown (rumdl)
      main.md:1:1: [MD041] First line in file should be a level 1 heading
      main.md:3:1: [MD012] Multiple consecutive blank lines between content [fixed]
      main.md:4:1: [MD012] Multiple consecutive blank lines between content [fixed]

      Fixed: Fixed 2/3 issues in 1 file (3ms)
      lint Markdown (rumdl)
      main.md:1:1: [MD041] First line in file should be a level 1 heading

      Issues: Found 1 issues in 1 file (2ms)
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
