Feature: sequential runs continue after a failure

  Scenario: the first test fails
    Given a file "trident.json" with content
      """
      {
        "tests": [
          { "name": "one", "command": "echo one failed && exit 3" },
          { "name": "two", "command": "echo two done" },
          { "name": "three", "command": "echo three failed && exit 4" }
        ]
      }
      """
    When executing "trident test --sequential --show=output"
    Then it prints
      """
      test all (one)
      one failed
      test all (two)
      two done
      test all (three)
      three failed
      """
    And the exit code is 4
