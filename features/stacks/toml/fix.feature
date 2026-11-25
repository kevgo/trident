Feature: fix TOML

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """

  Scenario: valid TOML
    Given a file "main.toml" with unix newlines
      """
      key = "value"
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      fix TOML (Taplo)
      """
    And the exit code is 0
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted TOML
    Given a file "main.toml" with unix newlines
      """
      key =     "value"
      """
    And a file "other.toml" with unix newlines
      """
      key =     "other"
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=2 excluded=0 files=["/tmp/.tmpWvNofv/7354843356311689787/main.toml", "/tmp/.tmpWvNofv/7354843356311689787/other.toml"] cwd="/tmp/.tmpWvNofv/7354843356311689787"
      """
    And the exit code is 0
    And file "main.toml" now has content
      """
      key = "value"
      """
    And file "other.toml" now has content
      """
      key = "other"
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid TOML
    Given a file "main.toml" with unix newlines
      """
      key = "
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpS2O3Z5/10565799516198748315/main.toml"] cwd="/tmp/.tmpS2O3Z5/10565799516198748315"
      error: invalid TOML
        ┌─ /tmp/.tmpS2O3Z5/10565799516198748315/main.toml:1:7
        │
      1 │ key = "
        │       ^ unexpected token

      error: invalid TOML
        ┌─ /tmp/.tmpS2O3Z5/10565799516198748315/main.toml:1:8
        │
      1 │   key = "
        │ ╭───────^
      2 │ │
        │ ╰^ expected value

      ERROR operation failed error=some files were not formatted due to syntax errors
      """
    And the exit code is 1
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
