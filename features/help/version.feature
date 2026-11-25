Feature: Displaying the version

  Scenario Outline: help command
    When executing "trident <ARG>"
    Then it prints
      """
      trident 0.1.0
      """
    And the exit code is 0
    And it runs no Git commands

    Examples:
      | ARG       |
      | --version |
      | -V        |
