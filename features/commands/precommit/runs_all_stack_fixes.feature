Feature: "trident precommit" runs all fixes for the changed files

  Scenario: runs Tikibase even though its config file is not changed
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      tikibase 0.6.2
      """
    And a file "tikibase.json" with unix newlines
      """
      {
        "ignore": [
          "run-that-app"
        ],
        "bidiLinks": false
      }
      """
    And a file "one.md" with unix newlines
      """
      # One

      text
      """
    And I ran "git add -A"
    And I ran "git commit -m original"
    And I change file "one.md" to
      """
      # New   one

      text
      """
    And I ran "git add one.md"
    When executing "trident precommit --show=output"
    Then it prints the lines
      """
      fix Markdown (tikibase)
      fix Markdown (rumdl)
      """
    And the exit code is 0
    And file "one.md" now has content
      """
      # New one

      text
      """
    And the staged changes are
      """
      diff --git a/one.md b/one.md
      index b3e70d0..4168e2b 100644
      --- a/one.md
      +++ b/one.md
      @@ -1,3 +1,3 @@
      -# One
      +# New one
       text
      """
    And there are no unstaged changes
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      git add -- one.md
      """
