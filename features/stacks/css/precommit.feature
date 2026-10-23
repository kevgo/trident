Feature: precommit CSS

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      """

  Scenario: valid CSS
    Given a file "main.css" with content
      """
      .foo {
      \tcolor: red;
      }
      """
    And I ran "git add main.css"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.css" is unchanged

  Scenario: unformatted CSS
    Given a file "main.css" with content
      """
      .foo {
        color : red ;
      }
      """
    And a file "other.css" with content
      """
      .bar {
        color : blue ;
      }
      """
    And I ran "git add main.css other.css"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the exit code is 0
    And file "main.css" now has content
      """
      .foo {
      \tcolor: red;
      }
      """
    And file "other.css" now has content
      """
      .bar {
      \tcolor: blue;
      }
      """

  Scenario: invalid CSS
    Given a file "main.css" with content
      """
      .foo {
        col
      }
      """
    And I ran "git add main.css"
    When executing "trident precommit"
    Then it prints the block
      """
      Found 2 errors.
      """
    And the exit code is 0
    And file "main.css" is unchanged
