Feature: lint YML

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      prettier 3.7.0
      """

  Scenario: valid YML
    Given a file "main.yml" with content
      """
      key: value
      """
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.yml" is unchanged

  Scenario: unformatted YML
    Given a file "main.yml" with content
      """
      key:     value
      """
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.yml" is unchanged

  Scenario: invalid YML
    Given a file "main.yml" with content
      """
      key: "
      """
    When executing "trident lint --show=output"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.yml" is unchanged
