Feature: precommit TOML

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """

  Scenario: valid TOML
    Given a file "main.toml" with unix newlines
      """
      key = "value"
      """
    And I ran "git add main.toml"
    When executing "trident precommit --show=output"
    Then it prints the block
      """
      fix TOML (Taplo)
      """
    And the exit code is 0
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames
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
    And I ran "git add main.toml other.toml"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=2 excluded=0 files=["/tmp/.tmpThmpIY/890595082096894755/main.toml", "/tmp/.tmpThmpIY/890595082096894755/other.toml"] cwd="/tmp/.tmpThmpIY/890595082096894755"
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
      git status --porcelain=v1 -z --no-renames
      git add -- main.toml other.toml
      """

  Scenario: invalid TOML
    Given a file "main.toml" with unix newlines
      """
      key = "
      """
    And I ran "git add main.toml"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpc1C0CO/11520627469013034025/main.toml"] cwd="/tmp/.tmpc1C0CO/11520627469013034025"
      error: invalid TOML
        ┌─ /tmp/.tmpc1C0CO/11520627469013034025/main.toml:1:7
        │
      1 │ key = "
        │       ^ unexpected token

      error: invalid TOML
        ┌─ /tmp/.tmpc1C0CO/11520627469013034025/main.toml:1:8
        │
      1 │   key = "
        │ ╭───────^
      2 │ │
        │ ╰^ expected value

      ERROR operation failed error=some files were not formatted due to syntax errors
      """
    And the exit code is 0
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames
      """
