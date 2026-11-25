Feature: "trident precommit" does not format files with unstaged changes

  Scenario: precommit an unstaged file
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """
    And a file "file.md" with unix newlines
      """
      line 1
      """
    And I ran "git add -A"
    And I ran "git commit -m original"
    And I change file "file.md" to
      """
      line   1
      """
    When executing "trident precommit"
    Then it prints nothing to STDOUT
    And there are no staged changes
    And the unstaged changes are
      """
      diff --git a/file.md b/file.md
      index 89b24ec..9ad9c0b 100644
      --- a/file.md
      +++ b/file.md
      @@ -1 +1 @@
      -line 1
      +line   1
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      """
