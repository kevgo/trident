Feature: fix Cucumber

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      ghokin 3.10.0
      """

  Scenario: valid Cucumber
    Given a file "main.feature" with content
      """
      Feature: foo

        Scenario: bar
          Given a step
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Cucumber (Ghokin)
      "main.feature" formatted
      """
    And the exit code is 0
    And file "main.feature" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Cucumber
    Given a file "main.feature" with content
      """
      Feature:   foo

        Scenario:   bar
          Given   a step
      """
    And a file "other.feature" with content
      """
      Feature:   foo2

        Scenario:   bar2
          Given   another step
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Cucumber (Ghokin)
      "main.feature" formatted
      "other.feature" formatted
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

  Scenario: invalid Cucumber
    Given a file "main.feature" with content
      """
      Feat
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Cucumber (Ghokin)
      Parser errors:
      (1:1): expected: #EOF, #Language, #TagLine, #FeatureLine, #Comment, #Empty, got 'Feat'
      """
    And the exit code is 1
    And file "main.feature" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
