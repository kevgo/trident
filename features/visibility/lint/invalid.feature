Feature: lint multiple stacks with invalid code

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
    And a file "main.css" with unix newlines
      """
      .foo {
      """
    And a file "main.ts" with unix newlines
      """
      console.log("
      """

  Scenario: --show=output
    When executing "trident lint --show=output"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 3 tasks
      """
    And it prints the lines
      """
      lint Python (Ruff)
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
      lint CSS (Biome)
      Checked 1 file in 864µs. No fixes applied.
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `}` but instead the file ends

          1 │ .foo {
        > 2 │
            │

        i the file ends here

          1 │ .foo {
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      lint TypeScript (Biome)
      Checked 1 file in 1705µs. No fixes applied.
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × unterminated string literal

        > 1 │ console.log("
            │             ^
          2 │

        i

        > 1 │ console.log("
            │             ^
          2 │

        i The closing quote must be on the same line.


      main.ts:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `)` but instead the file ends

          1 │ console.log("
        > 2 │
            │

        i the file ends here

          1 │ console.log("
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      """
    And it prints the lines
      """
      lint Python (Ruff)
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
      lint CSS (Biome)
      Checked 1 file in 864µs. No fixes applied.
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `}` but instead the file ends

          1 │ .foo {
        > 2 │
            │

        i the file ends here

          1 │ .foo {
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      lint TypeScript (Biome)
      Checked 1 file in 1705µs. No fixes applied.
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × unterminated string literal

        > 1 │ console.log("
            │             ^
          2 │

        i

        > 1 │ console.log("
            │             ^
          2 │

        i The closing quote must be on the same line.


      main.ts:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `)` but instead the file ends

          1 │ console.log("
        > 2 │
            │

        i the file ends here

          1 │ console.log("
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      """
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: --show=verbose
    When executing "trident lint --show=verbose"
    Then it prints to STDERR
      """
      1 CSS, 1 Python, 1 TypeScript, 1 other
      running 3 tasks
      """
    And it prints the lines
      """
      lint Python (Ruff)
      /home/kevlar/.run-that-app/apps/ruff@0.15.16/ruff-aarch64-unknown-linux-gnu/ruff check main.py
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
      lint CSS (Biome)
      /home/kevlar/.run-that-app/apps/biome@2.4.0/biome lint main.css
      Checked 1 file in 670µs. No fixes applied.
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `}` but instead the file ends

          1 │ .foo {
        > 2 │
            │

        i the file ends here

          1 │ .foo {
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      lint TypeScript (Biome)
      /home/kevlar/.run-that-app/apps/biome@2.4.0/biome lint main.ts
      Checked 1 file in 961µs. No fixes applied.
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × unterminated string literal

        > 1 │ console.log("
            │             ^
          2 │

        i

        > 1 │ console.log("
            │             ^
          2 │

        i The closing quote must be on the same line.


      main.ts:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `)` but instead the file ends

          1 │ console.log("
        > 2 │
            │

        i the file ends here

          1 │ console.log("
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      """
    And it prints the lines
      """
      lint Python (Ruff)
      /home/kevlar/.run-that-app/apps/ruff@0.15.16/ruff-aarch64-unknown-linux-gnu/ruff check main.py
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
      lint CSS (Biome)
      /home/kevlar/.run-that-app/apps/biome@2.4.0/biome lint main.css
      Checked 1 file in 670µs. No fixes applied.
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `}` but instead the file ends

          1 │ .foo {
        > 2 │
            │

        i the file ends here

          1 │ .foo {
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      lint TypeScript (Biome)
      /home/kevlar/.run-that-app/apps/biome@2.4.0/biome lint main.ts
      Checked 1 file in 961µs. No fixes applied.
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × unterminated string literal

        > 1 │ console.log("
            │             ^
          2 │

        i

        > 1 │ console.log("
            │             ^
          2 │

        i The closing quote must be on the same line.


      main.ts:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `)` but instead the file ends

          1 │ console.log("
        > 2 │
            │

        i the file ends here

          1 │ console.log("
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      """
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: --show=names
    When executing "trident lint --show=names"
    Then it prints nothing to STDERR
    And it prints the lines
      """
      lint Python (Ruff)
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
      lint CSS (Biome)
      Checked 1 file in 567µs. No fixes applied.
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `}` but instead the file ends

          1 │ .foo {
        > 2 │
            │

        i the file ends here

          1 │ .foo {
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      lint TypeScript (Biome)
      Checked 1 file in 964µs. No fixes applied.
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × unterminated string literal

        > 1 │ console.log("
            │             ^
          2 │

        i

        > 1 │ console.log("
            │             ^
          2 │

        i The closing quote must be on the same line.


      main.ts:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `)` but instead the file ends

          1 │ console.log("
        > 2 │
            │

        i the file ends here

          1 │ console.log("
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      """
    And it prints the lines
      """
      lint Python (Ruff)
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
      lint CSS (Biome)
      Checked 1 file in 567µs. No fixes applied.
      Found 1 error.
      main.css:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `}` but instead the file ends

          1 │ .foo {
        > 2 │
            │

        i the file ends here

          1 │ .foo {
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      lint TypeScript (Biome)
      Checked 1 file in 964µs. No fixes applied.
      Found 2 errors.
      main.ts:1:13 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × unterminated string literal

        > 1 │ console.log("
            │             ^
          2 │

        i

        > 1 │ console.log("
            │             ^
          2 │

        i The closing quote must be on the same line.


      main.ts:2:1 parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × expected `)` but instead the file ends

          1 │ console.log("
        > 2 │
            │

        i the file ends here

          1 │ console.log("
        > 2 │
            │


      lint ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

        × Some errors were emitted while running checks.


      """
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: --show=failed
    When executing "trident lint --show=failed"
    Then it prints nothing to STDERR
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
    And it prints the block
      """
      invalid-syntax: missing closing quote in string literal
       --> main.py:1:7
      """
    And it does not print any of these lines
      """
      lint CSS (Biome)
      lint TypeScript (Biome)
      """
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
