Feature: lint multiple stacks with unformatted files

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """
    And a file "main.py" with content
      """
      print(  "hello"  )
      """
    And a file "main.css" with content
      """
      .foo {
        color : red ;
      }
      """
    And a file "main.ts" with content
      """
      console.log(  "hello"  );
      """

  Scenario: --show=output
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 3 tools
      """
    And it prints the block
      """
      lint Python (ruff)
      All checks passed!
      """
    And it prints the block
      """
      lint CSS (Biome)
      """
    And it prints the block
      """
      lint TypeScript (Biome)
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: --show=verbose
    When executing "trident lint --show=verbose"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 3 tools
      """
    And it prints the lines matching
      """
      lint Python \(ruff\)
      ruff(.exe)? check main.py
      All checks passed!
      """
    And it prints the lines matching
      """
      lint CSS \(Biome\)
      biome(.exe)? lint main.css
      """
    And it prints the lines matching
      """
      lint TypeScript \(Biome\)
      biome(.exe)? lint main.ts
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: --show=names
    When executing "trident lint --show=names"
    Then it prints nothing to STDERR
    And it prints only these lines in any order
      """
      lint Python (ruff)
      lint TypeScript (Biome)
      lint CSS (Biome)
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: --show=failed
    When executing "trident lint --show=failed"
    Then it prints nothing to STDERR
    And it prints nothing to STDOUT
    And the exit code is 0
    And all files are unchanged
