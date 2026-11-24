Feature: lint Cucumber

  Background:
    Given a file "run-that-app" with unix newlines
      """
      delete-empty-folders 0.0.2
      gherkin-lint 4.2.4
      node 26.4.0
      """
    And a file ".gherkin-lintrc" with unix newlines
      """
      {
        "allowed-tags": [ "on", { "tags": ["@online"] } ],
        "file-name": [ "on", { "style": "snake_case" } ],
        "indentation": [
          "on",
          {
            "Feature": 0,
            "Background": 2,
            "Scenario": 2,
            "Step": 4,
            "Examples": 4,
            "example": 6,
            "given": 4,
            "when": 4,
            "then": 4,
            "and": 4,
            "but": 4,
            "feature tag": 0,
            "scenario tag": 2
          }
        ],
        "keywords-in-logical-order": "on",
        "new-line-at-eof": [ "on", "yes" ],
        "no-background-only-scenario": "on",
        "no-dupe-scenario-names": [ "on", "in-feature" ],
        "no-duplicate-tags": "on",
        "no-empty-background": "on",
        "no-empty-file": "on",
        "no-examples-in-scenarios": "on",
        "no-files-without-scenarios": "on",
        "no-homogenous-tags": "on",
        "no-multiple-empty-lines": "on",
        "no-partially-commented-tag-lines": "on",
        "no-scenario-outlines-without-examples": "on",
        "no-superfluous-tags": "on",
        "no-trailing-spaces": "on",
        "no-unnamed-features": "on",
        "no-unused-variables": "on",
        "one-space-between-tags": "on",
        "use-and": "on"
      }
      """

  Scenario: valid Cucumber
    Given a file "features/one.feature" with unix newlines
      """
      Feature: one

        Scenario: one
          Given a step
      """
    When executing "trident lint --show=output"
    Then it prints
      """
      lint Cucumber (gherkin-lint)
      """
    And the exit code is 0
    And file "features/one.feature" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Cucumber
    Given a file "features/one.feature" with unix newlines
      """
      Feature: one

        Scenario: one
                  Given a step
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Cucumber (gherkin-lint)
        4    Wrong indentation for "given", expected indentation level of 4, but got 12    indentation
      """
    And the exit code is 1
    And file "features/one.feature" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid Cucumber
    Given a file "features/one.feature" with unix newlines
      """
      Feat
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Cucumber (gherkin-lint)
        1    (1:1): expected: #EOF, #Language, #TagLine, #FeatureLine, #Comment, #Empty, got 'Feat'    unexpected-error
      """
    And the exit code is 1
    And file "features/one.feature" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
