Feature: "trident precommit" formats and stages files with partially staged changes

  Scenario: precommit partially staged changes
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """
    And a file "file.md" with content
      """
      line 1 old

      line 2 old
      """
    And I ran "git add -A"
    And I ran "git commit -m original"
    And I change file "file.md" to
      """
      line   1 new

      line 2 old
      """
    And I ran "git add file.md"
    And I change file "file.md" to
      """
      line   1 new

      line   2 new
      """
    When executing "trident precommit"
    Then the staged changes are
      """
      diff --git a/file.md b/file.md
      index 9c82e17..5103d60 100644
      --- a/file.md
      +++ b/file.md
      @@ -1,3 +1,3 @@
      -line 1 old
      +line 1 new
      -line 2 old
      +line 2 new
      """
    And there are no unstaged changes
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z
      git add -- file.md
      """
