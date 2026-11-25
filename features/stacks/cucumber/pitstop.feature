Feature: pitstop Cucumber

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      gherkin-lint 4.2.4
      ghokin 3.10.0
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

  Scenario: unformatted Cucumber
    Given a file "main.feature" with unix newlines
      """
      Feature:   foo

        Scenario:   bar
          Given   a step
      """
    And a file "other.feature" with unix newlines
      """
      Feature:   foo2

        Scenario:   bar2
          Given   another step
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix Cucumber (Ghokin)
      lint Cucumber (gherkin-lint)
      """
    And the exit code is 0
    And file "main.feature" now has content
      """
      Feature: foo

        Scenario: bar
          Given a step
      """
    And file "other.feature" now has content
      """
      Feature: foo2

        Scenario: bar2
          Given another step
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Cucumber with lint error
    Given a file "main.feature" with unix newlines
      """
      Feature:    foo

          Given a step
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix Cucumber (Ghokin)
      lint Cucumber (gherkin-lint)
        1    Feature file does not have any Scenarios    no-files-without-scenarios
      """
    And the exit code is 1
    And file "main.feature" now has content
      """
      Feature: foo

        Given a step
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
