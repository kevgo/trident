Feature: unsafe-fix TOML

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """
    And a file "main.toml" with unix newlines
      """
      [package
      name="demo"
      version   = "0.1.0"
      """

  Scenario: fix-unsafe
    When executing "trident fix-unsafe --show=output"
    Then it prints the lines
      """
      unsafe-fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpFv7cCR/1763452318113921398/main.toml"] cwd="/tmp/.tmpFv7cCR/1763452318113921398"
      error: invalid TOML
        ┌─ /tmp/.tmpFv7cCR/1763452318113921398/main.toml:1:9
        │
      1 │   [package
        │ ╭────────^
      2 │ │ name="demo"
        │ ╰^ expected "]"

      """
    And the exit code is 0
    And file "main.toml" now has content
      """
      [package
      name="demo"
      version = "0.1.0"
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: fix
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpCf4rIX/6847348958844464047/main.toml"] cwd="/tmp/.tmpCf4rIX/6847348958844464047"
      error: invalid TOML
        ┌─ /tmp/.tmpCf4rIX/6847348958844464047/main.toml:1:9
        │
      1 │   [package
        │ ╭────────^
      2 │ │ name="demo"
        │ ╰^ expected "]"

      ERROR operation failed error=some files were not formatted due to syntax errors
      """
    And the exit code is 1
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
