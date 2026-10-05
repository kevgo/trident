Feature: fix multiple stacks

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
      p {
        color : red ;
      }
      """
    And a file "main.ts" with content
      """
      console.log(  "hello"  );
      """

  Scenario: --show=output
    When executing "trident fix --show=output"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 5 tools
      """
    And it prints the block
      """
      fix TypeScript (Biome)
      """
    And it prints the block
      """
      fix CSS (Biome)
      """
    And it prints the block
      """
      fix Python (ruff)
      All checks passed!
      """
    And it prints the block
      """
      format Python (ruff)
      1 file reformatted
      """
    And file "main.css" now has content
      """
      p {
      \tcolor: red;
      }
      """
    And file "main.ts" now has content
      """
      console.log("hello");
      """
    And file "main.py" now has content
      """
      print("hello")
      """

  Scenario: --show=verbose
    When executing "trident fix --show=verbose"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 5 tools
      """
    And it prints the lines matching
      """
      fix TypeScript \(Biome\)
      biome(.exe)? format --write main.ts
      """
    And it prints the lines matching
      """
      fix CSS \(Biome\)
      biome(.exe)? format --write main.css
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
      1 file reformatted
      """
    And file "main.css" now has content
      """
      p {
      \tcolor: red;
      }
      """
    And file "main.ts" now has content
      """
      console.log("hello");
      """
    And file "main.py" now has content
      """
      print("hello")
      """

  Scenario: --show=names
    When executing "trident fix --show=names"
    Then it does not print
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 4 tools
      """
    And it prints only these lines in any order
      """
      delete empty folders
      fix Python (ruff)
      format Python (ruff)
      fix TypeScript (Biome)
      fix CSS (Biome)
      """
    And file "main.css" now has content
      """
      p {
      \tcolor: red;
      }
      """
    And file "main.ts" now has content
      """
      console.log("hello");
      """
    And file "main.py" now has content
      """
      print("hello")
      """

  Scenario: --show=failed
    When executing "trident fix --show=failed"
    Then it prints nothing to STDOUT
    And file "main.css" now has content
      """
      p {
      \tcolor: red;
      }
      """
    And file "main.ts" now has content
      """
      console.log("hello");
      """
    And file "main.py" now has content
      """
      print("hello")
      """
