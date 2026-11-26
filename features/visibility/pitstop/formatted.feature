Feature: pitstop multiple good stacks

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
    And a file "main.css" with unix newlines
      """
      .foo {
      \tcolor: red;
      }
      """
    And a file "main.ts" with unix newlines
      """
      console.log("hello");
      """

  Scenario: --show=output
    When executing "trident pitstop --show=output"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 8 tasks
      """
    And it prints the lines
      """
      fix Python (Ruff)
      All checks passed!
      format Python (Ruff)
      1 file left unchanged
      lint Python (Ruff)
      All checks passed!
      """
    And it prints the lines
      """
      fix CSS (Biome)
      lint CSS (Biome)
      """
    And it prints the lines
      """
      fix TypeScript (Biome)
      lint TypeScript (Biome)
      """
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: --show=verbose
    When executing "trident pitstop --show=verbose"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 8 tasks
      """
    And it prints the lines matching
      """
      fix Python \(Ruff\)
      ruff(.exe)? check --fix main.py$
      All checks passed!
      format Python \(Ruff\)
      ruff(.exe)? format main.py$
      1 file left unchanged
      lint Python \(Ruff\)
      ruff(.exe)? check main.py$
      All checks passed!
      """
    And it prints the lines matching
      """
      fix CSS \(Biome\)
      biome format --write main.css$
      lint CSS \(Biome\)
      biome lint main.css$
      """
    And it prints the lines matching
      """
      fix TypeScript \(Biome\)
      biome format --write main.ts$
      lint TypeScript \(Biome\)
      biome lint main.ts$
      """
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: --show=names
    When executing "trident pitstop --show=names"
    Then it prints only these lines in any order
      """
      fix all (delete empty folders)
      fix Python (Ruff)
      format Python (Ruff)
      lint Python (Ruff)
      fix CSS (Biome)
      lint CSS (Biome)
      fix TypeScript (Biome)
      lint TypeScript (Biome)
      """
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: --show=failed
    When executing "trident pitstop --show=failed"
    Then it prints nothing to STDOUT
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
