Feature: wrong CLI command

  Scenario: calling a non-existing subcommand
    When executing "trident zonk"
    Then it prints the lines matching
      """
      error: unrecognized subcommand 'zonk'

      Usage: trident(.exe)? <COMMAND>

      For more information, try '--help'.
      """
    And the exit code is 1
