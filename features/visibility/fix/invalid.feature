Feature: fix multiple stacks with invalid code

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      ruff 0.15.16
      """
    And a file "main.py" with content
      """
      print("
      """
    And a file "main.css" with content
      """
      .foo {
      """
    And a file "main.ts" with content
      """
      console.log("
      """

  Scenario: --show=output
    When executing "trident fix --show=output"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 5 tools
      """
    And it prints the lines
      """
      fix CSS (Biome)
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      """
    And it prints the lines
      """
      fix TypeScript (Biome)
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      """
    # no python error message here
    And all files are unchanged

  Scenario: --show=verbose
    When executing "trident fix --show=verbose"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 5 tools
      """
    And it prints the lines
      """
      fix CSS (Biome)
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      """
    And it prints the lines
      """
      fix TypeScript (Biome)
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      """
    # no python error message here
    And all files are unchanged

  Scenario: --show=names
    When executing "trident fix --show=names"
    Then it prints nothing to STDERR
    And it prints the lines
      """
      invalid-syntax: missing closing quote in string literal
       --> main.py:1:7
        |
      1 | print("
        |       ^
        |

      invalid-syntax: unexpected EOF while parsing
       --> main.py:2:1
        |
      1 | print("
        |        ^
        |

      Found 2 errors.
      """
    And it prints the lines
      """
      fix CSS (Biome)
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      """
    And it prints the lines
      """
      fix TypeScript (Biome)
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      """
    And all files are unchanged

  Scenario: --show=failed
    When executing "trident fix --show=failed"
    Then it prints nothing to STDERR
    And it does not print any of these lines
      """
      fix CSS (Biome)
      fix TypeScript (Biome)
      fix Python (Ruff)
      format Python (Ruff)
      """
    And it prints the block
      """
      invalid-syntax: missing closing quote in string literal
       --> main.py:1:7
        |
      1 | print("
        |       ^
        |

      invalid-syntax: unexpected EOF while parsing
       --> main.py:2:1
        |
      1 | print("
        |        ^
        |

      Found 2 errors.
      """
    And it prints the block
      """
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      """
    And it prints the block
      """
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
      """
    And all files are unchanged
