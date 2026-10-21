Feature: full runs all fixes, lints, and tests

  Scenario: runs all fixes, lints, and tests on all files
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """
    And a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "unit tests", "command": "echo unit" }
        ]
      }
      """
    And a file "main.json" with content
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
      running 7 tools
      """
    And it prints the lines
      """
      fix JSON (Biome)
      """
    And it prints the block
      """
      format Python (ruff)
      1 file reformatted
      """
    And it prints the block
      """
      lint Python (ruff)
      All checks passed!
      """
    And it prints the block
      """
      unit tests
      unit
      """
    And file "main.json" now has content
      """
      { "key": "value" }
      """
    And file "main.py" now has content
      """
      print("hello")
      """
    And the exit code is 0

  Scenario: commands.full.test runs those tests by default
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "biome": { "enabled": false }
        },
        "tests": [
          { "name": "unit", "command": "echo unit" },
          { "name": "cuke", "command": "echo cuke" }
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
      running 2 tools
      """
    And it prints the block
      """
      unit
      unit
      """
    And it does not print
      """
      cuke
      """
    And the exit code is 0
