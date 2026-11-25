Feature: lint TOML

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
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint TOML (Taplo)
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
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint TOML (Taplo)
       INFO taplo:lint_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpNfl7NR/2593806222753098134/main.toml"] cwd="/tmp/.tmpNfl7NR/2593806222753098134"
      """
    And the exit code is 0
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid TOML
    Given a file "main.toml" with unix newlines
      """
      key = "
      """
    And a file "other.toml" with unix newlines
      """
      other = "
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint TOML (Taplo)
       INFO taplo:lint_files:collect_files: found files total=2 excluded=0 files=["/tmp/.tmpFE4A6A/9250336155963017541/main.toml", "/tmp/.tmpFE4A6A/9250336155963017541/other.toml"] cwd="/tmp/.tmpFE4A6A/9250336155963017541"
      error: invalid TOML
        ┌─ /tmp/.tmpFE4A6A/9250336155963017541/main.toml:1:7
        │
      1 │ key = "
        │       ^ unexpected token

      error: invalid TOML
        ┌─ /tmp/.tmpFE4A6A/9250336155963017541/main.toml:1:8
        │
      1 │   key = "
        │ ╭───────^
      2 │ │
        │ ╰^ expected value

      ERROR taplo:lint_files: invalid file error=syntax errors found path="/tmp/.tmpFE4A6A/9250336155963017541/main.toml"
      error: invalid TOML
        ┌─ /tmp/.tmpFE4A6A/9250336155963017541/other.toml:1:9
        │
      1 │ other = "
        │         ^ unexpected token

      error: invalid TOML
        ┌─ /tmp/.tmpFE4A6A/9250336155963017541/other.toml:1:10
        │
      1 │   other = "
        │ ╭─────────^
      2 │ │
        │ ╰^ expected value

      ERROR taplo:lint_files: invalid file error=syntax errors found path="/tmp/.tmpFE4A6A/9250336155963017541/other.toml"
      ERROR operation failed error=some files were not valid
      """
    And the exit code is 1
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
