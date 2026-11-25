@online
Feature: install all TOML tools

  Scenario: not installed
    Given a file "main.toml" with unix newlines
      """
      key =      "value"
      """
    When executing "trident fix --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/tamasfe/taplo/releases/latest) ... ok
      """
    And it prints the lines
      """
      fix all (delete empty folders)
      fix TOML (Taplo)
       INFO taplo:format_files:collect_files: found files total=1 excluded=0 files=["/tmp/.tmpTbVklN/7837115179257223702/main.toml"] cwd="/tmp/.tmpTbVklN/7837115179257223702"
      """
    And the exit code is 0
    And file "main.toml" now has content
      """
      key = "value"
      """
    And file "run-that-app" now has an additional line matching
      """
      delete-empty-folders \d+\.\d+\.\d+
      taplo \d+\.\d+\.\d+
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
