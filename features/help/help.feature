@this
Feature: Displaying help

  Scenario: no command given
    When executing "trident"
    Then it prints the lines matching
      """
      error: 'trident(\.exe)?' requires a subcommand but one was not provided
        \[subcommands: ci, init:claude, init:config, init:githook, fix, fix-unsafe, full, lint, postgenerate, pitstop, postedit, precommit, test, update:tools, help\]

      Usage: trident(\.exe)? <COMMAND>

      For more information, try '--help'.
      """
    And the exit code is 1

  Scenario: help command
    When executing "trident help"
    Then it prints the lines matching
      """
      The all-in-one DevEx tool.

      Usage: trident(\.exe)? <COMMAND>

      Commands:
        ci            Runs all fixes, lints, and tests on CI
        init:claude   Embed into claude-compatible coding agents
        init:config   Create the Trident configuration file
        init:githook  Install the Git pre-commit hook
        fix           Apply safe code quality fixes
        fix-unsafe    Apply advanced fixes that might change behavior
        full          Run all lints, fixes, and tests on all files
        lint          Find code quality issues \[alias: postgenerate\]
        pitstop       Fix and lint the current work
        postedit      Lint uncommitted changes
        precommit     Fix staged files before committing, never fails
        test          Run all tests in parallel
        update:tools  Update third-party tools
        help          Print this message or the help of the given subcommand\(s\)

      Options:
        -h, --help     Print help
        -V, --version  Print version
      """
    And the exit code is 0

  Scenario: --help flag
    When executing "trident --help"
    Then it prints the lines matching
      """
      The all-in-one DevEx tool.

      Usage: trident(\.exe)? <COMMAND>

      Commands:
        ci            Runs all fixes, lints, and tests on CI
        init:claude   Embed into claude-compatible coding agents
        init:config   Create the Trident configuration file
        init:githook  Install the Git pre-commit hook
        fix           Apply safe code quality fixes
        fix-unsafe    Apply advanced fixes that might change behavior
        full          Run all lints, fixes, and tests on all files
        lint          Find code quality issues \[alias: postgenerate\]
        pitstop       Fix and lint the current work
        postedit      Lint uncommitted changes
        precommit     Fix staged files before committing, never fails
        test          Run all tests in parallel
        update:tools  Update third-party tools
        help          Print this message or the help of the given subcommand\(s\)

      Options:
        -h, --help     Print help
        -V, --version  Print version
      """
    And the exit code is 0

  Scenario: -h flag
    When executing "trident -h"
    Then it prints the lines matching
      """
      The all-in-one DevEx tool.

      Usage: trident(\.exe)? <COMMAND>

      Commands:
        ci            Runs all fixes, lints, and tests on CI
        init:claude   Embed into claude-compatible coding agents
        init:config   Create the Trident configuration file
        init:githook  Install the Git pre-commit hook
        fix           Apply safe code quality fixes
        fix-unsafe    Apply advanced fixes that might change behavior
        full          Run all lints, fixes, and tests on all files
        lint          Find code quality issues \[alias: postgenerate\]
        pitstop       Fix and lint the current work
        postedit      Lint uncommitted changes
        precommit     Fix staged files before committing, never fails
        test          Run all tests in parallel
        update:tools  Update third-party tools
        help          Print this message or the help of the given subcommand\(s\)

      Options:
        -h, --help     Print help
        -V, --version  Print version
      """
    And the exit code is 0

  Scenario: help for a subcommand
    When executing "trident help lint"
    Then it prints the lines matching
      """
      Find code quality issues

      Usage: trident(\.exe)? lint \[OPTIONS\]

      Options:
            --show <SHOW>
                how much output to display

                Possible values:
                - failed:  only output of failed commands
                - names:   command names and output of failed commands
                - output:  command names and output of all commands
                - verbose: command lines and output of all commands

            --scope <SCOPE>
                files to apply the operation to

                Possible values:
                - uncommitted: uncommitted files
                - branch:      files changed on the current branch
                - all:         all files in the current directory

        -h, --help
                Print help \(see a summary with '-h'\)
      """
    And the exit code is 0

  Scenario: help for the ci command
    When executing "trident help ci"
    Then it prints the lines matching
      """
      Runs all fixes, lints, and tests on CI

      Usage: trident(\.exe)? ci \[OPTIONS\]

      Options:
            --show <SHOW>
                how much output to display

                Possible values:
                - failed:  only output of failed commands
                - names:   command names and output of failed commands
                - output:  command names and output of all commands
                - verbose: command lines and output of all commands

            --scope <SCOPE>
                files to apply the operation to

                Possible values:
                - uncommitted: uncommitted files
                - branch:      files changed on the current branch
                - all:         all files in the current directory

            --test <NAME>
                names of tests to run, joined with \+

        -h, --help
                Print help \(see a summary with '-h'\)
      """
    And the exit code is 0

  Scenario: help for the full command
    When executing "trident help full"
    Then it prints the lines matching
      """
      Run all lints, fixes, and tests on all files

      Usage: trident(\.exe)? full \[OPTIONS\]

      Options:
            --show <SHOW>
                how much output to display

                Possible values:
                - failed:  only output of failed commands
                - names:   command names and output of failed commands
                - output:  command names and output of all commands
                - verbose: command lines and output of all commands

            --test <NAME>
                names of tests to run, joined with \+

        -h, --help
                Print help \(see a summary with '-h'\)
      """
    And the exit code is 0

  Scenario: help for the pitstop command
    When executing "trident help pitstop"
    Then it prints the lines matching
      """
      Fix and lint the current work

      Usage: trident(\.exe)? pitstop \[OPTIONS\]

      Options:
            --show <SHOW>
                how much output to display

                Possible values:
                - failed:  only output of failed commands
                - names:   command names and output of failed commands
                - output:  command names and output of all commands
                - verbose: command lines and output of all commands

            --scope <SCOPE>
                files to apply the operation to

                Possible values:
                - uncommitted: uncommitted files
                - branch:      files changed on the current branch
                - all:         all files in the current directory

            --test <NAME>
                names of tests to run, joined with \+

        -h, --help
                Print help \(see a summary with '-h'\)
      """
    And the exit code is 0

  Scenario: help for the fix command
    When executing "trident help fix"
    Then it prints the lines matching
      """
      Apply safe code quality fixes

      Usage: trident(\.exe)? fix \[OPTIONS\]

      Options:
            --show <SHOW>
                how much output to display

                Possible values:
                - failed:  only output of failed commands
                - names:   command names and output of failed commands
                - output:  command names and output of all commands
                - verbose: command lines and output of all commands

            --scope <SCOPE>
                files to apply the operation to

                Possible values:
                - uncommitted: uncommitted files
                - branch:      files changed on the current branch
                - all:         all files in the current directory

        -h, --help
                Print help \(see a summary with '-h'\)
      """
    And the exit code is 0

  Scenario: help for the test command
    When executing "trident help test"
    Then it prints the lines matching
      """
      Run all tests in parallel

      Usage: trident(\.exe)? test \[OPTIONS\]

      Options:
            --show <SHOW>
                how much output to display

                Possible values:
                - failed:  only output of failed commands
                - names:   command names and output of failed commands
                - output:  command names and output of all commands
                - verbose: command lines and output of all commands

            --test <NAME>
                names of tests to run, joined with \+

        -h, --help
                Print help \(see a summary with '-h'\)
      """
    And the exit code is 0
