Feature: fix TOML

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      taplo 0.10.0
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
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=2 excluded=0 files=["/tmp/.tmpEeB1IL/8476872760046772247/main.toml", "/tmp/.tmpEeB1IL/8476872760046772247/other.toml"] cwd="/tmp/.tmpEeB1IL/8476872760046772247"
      lint TOML (Taplo)
       INFO taplo:lint_files:collect_files: found files total=2 excluded=0 files=["/tmp/.tmpEeB1IL/8476872760046772247/main.toml", "/tmp/.tmpEeB1IL/8476872760046772247/other.toml"] cwd="/tmp/.tmpEeB1IL/8476872760046772247"
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

  Scenario: unformatted TOML with lint errors
    Given a file "main.toml" with unix newlines
      """
      key = 1
      key = 2
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix all (delete empty folders)
      fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpz8aKv7/11012814068424406396/main.toml"] cwd="/tmp/.tmpz8aKv7/11012814068424406396"
      lint TOML (Taplo)
       INFO taplo:lint_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpz8aKv7/11012814068424406396/main.toml"] cwd="/tmp/.tmpz8aKv7/11012814068424406396"
      error: conflicting keys
        ┌─ /tmp/.tmpz8aKv7/11012814068424406396/main.toml:2:1
        │
      1 │ key = 1
        │ --- duplicate found here
      2 │ key = 2
        │ ^^^ duplicate key

      ERROR taplo:lint_files: invalid file error=semantic errors found path="/tmp/.tmpz8aKv7/11012814068424406396/main.toml"
      ERROR operation failed error=some files were not valid
      """
    And the exit code is 1
    And file "main.toml" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
