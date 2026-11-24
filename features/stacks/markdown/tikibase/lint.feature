Feature: lint a Tikibase

  Background:
    Given a file "run-that-app" with unix newlines
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      tikibase 0.6.2
      """
    And a file "tikibase.json" with unix newlines
      """
      {
        "ignore": [
          "run-that-app"
        ],
        "bidiLinks": true
      }
      """

  Scenario: valid Markdown
    Given a file "one.md" with unix newlines
      """
      # One

      also check out [Two](two.md)
      """
    And a file "two.md" with unix newlines
      """
      # Two

      also check out [One](one.md)
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Markdown (tikibase)
      """
    And it prints the block
      """
      lint Markdown (rumdl)
      """
    And the exit code is 0
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Markdown
    Given a file "one.md" with unix newlines
      """
      # One

      also check out [Two](two.md)
      """
    And a file "two.md" with unix newlines
      """
      # Two
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Markdown (tikibase)
      two.md:1  document is not connected to any other documents
      two.md:1  missing link to one.md
      """
    And the exit code is 2
    And all files are unchanged
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
    Then it prints the block
      """
      lint Markdown (tikibase)
      main.md:1  no title section
      """
    And it prints the block
      """
      lint Markdown (rumdl)
      main.md:1:1: [MD041] First line in file should be a level 1 heading
      """
    And the exit code is 1
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
