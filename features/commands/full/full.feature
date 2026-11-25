Feature: full runs all fixes, lints, and tests

  Scenario: runs all fixes, lints, and tests on all files
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """
    And a file "trident.json" with unix newlines
      """
      {
        "tests": [
          { "name": "unit", "command": "echo running unit tests" }
        ]
      }
      """
    And a file "main.json" with unix newlines
      """
      {  "key"  :  "value"  }
      """
    And a file "main.py" with content
      """
      print(  "hello"  )
      """
    When executing "trident full --show=output"
    Then it prints the lines to STDERR
      """
      2 JSON, 1 Python, 1 other
      running 7 tasks
      """
    And it prints the block
      """
      format Python (Ruff)
      1 file reformatted
      """
    And it prints the block
      """
      lint Python (Ruff)
      All checks passed!
      """
    And it prints the block
      """
      test all (unit)
      running unit tests
      """
    And it prints the lines
      """
      fix all (delete empty folders)
      fix Python (Ruff)
      All checks passed!
      fix JSON (Biome)
      Formatted 2 files in 312µs. Fixed 2 files.
      format Python (Ruff)
      1 file reformatted
      lint Python (Ruff)
      All checks passed!
      lint JSON (Biome)
      Checked 2 files in 655µs. No fixes applied.
      test all (unit)
      running unit tests
      """
    And the exit code is 0
    And file "main.json" now has content
      """
      { "key": "value" }
      """
    And file "main.py" now has content
      """
      print("hello")
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: commands.full.test runs those tests by default
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """
    And a file "trident.json" with unix newlines
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo running unit tests" },
          { "name": "cuke", "command": "echo running cuke tests" }
        ],
        "commands": {
          "full": {
            "test": ["unit"]
          }
        }
      }
      """
    When executing "trident full --show=output"
    Then it prints the lines to STDERR
      """
      1 JSON, 1 other
      running 2 tasks
      """
    And it prints the block
      """
      test all (unit)
      running unit tests
      """
    And it does not print
      """
      running cuke tests
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
