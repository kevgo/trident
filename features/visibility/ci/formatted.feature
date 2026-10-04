Feature: CI multiple formatted stacks

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """
    And a file "main.py" with content
      """
      print("hello")
      """
    And a file "main.css" with content
      """
      .foo {
      \tcolor: red;
      }
      """
    And a file "main.ts" with content
      """
      console.log("hello");
      """

  Scenario: --show=output
    When executing "trident ci --show=output"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 8 tools
      """
    And it prints the lines
      """
      fix CSS (Biome)
      """
    And it prints the lines
      """
      lint CSS (Biome)
      """
    And it prints the lines
      """
      fix TypeScript (Biome)
      """
    And it prints the lines
      """
      lint TypeScript (Biome)
      """
    And it prints the block
      """
      fix Python (ruff)
      All checks passed!
      """
    And it prints the block
      """
      format Python (ruff)
      1 file left unchanged
      """
    And it prints the block
      """
      lint Python (ruff)
      All checks passed!
      """
    And all files are unchanged
    And the exit code is 0

  Scenario: --show=verbose
    When executing "trident ci --show=verbose"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 8 tools
      """
    And it prints the lines
      """
      fix CSS (Biome)
      """
    And it prints the lines
      """
      lint CSS (Biome)
      """
    And it prints the lines
      """
      fix TypeScript (Biome)
      """
    And it prints the lines
      """
      lint TypeScript (Biome)
      """
    And it prints the lines matching
      """
      fix Python \(ruff\)
      ruff(.exe)? check --fix main.py
      All checks passed!
      """
    And it prints the lines matching
      """
      format Python \(ruff\)
      ruff(.exe)? format main.py
      1 file left unchanged
      """
    And it prints the lines matching
      """
      lint Python \(ruff\)
      ruff(.exe)? check main.py
      All checks passed!
      """
    And all files are unchanged
    And the exit code is 0

  Scenario: --show=names
    When executing "trident ci --show=names"
    Then it prints only these lines in any order
      """
      delete empty folders
      fix Python (ruff)
      format Python (ruff)
      lint Python (ruff)
      fix CSS (Biome)
      lint CSS (Biome)
      fix TypeScript (Biome)
      lint TypeScript (Biome)
      """
    And all files are unchanged
    And the exit code is 0

  Scenario: --show=failed
    When executing "trident ci --show=failed"
    Then it prints nothing to STDOUT
    And all files are unchanged
    And the exit code is 0
