Feature: "trident precommit" formats and stages files with fully staged changes

  Scenario: precommit a fully staged file
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """
    And a file "file.md" with content
      """
      # Original
      """
    And I ran "git add -A"
    And I ran "git commit -m original"
    And I change file "file.md" to
      """
      #     New
      """
    And I ran "git add file.md"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the staged changes are
      """
      diff --git a/file.md b/file.md
      index 6950014..e65f941 100644
      --- a/file.md
      +++ b/file.md
      @@ -1 +1 @@
      -# Original
      +# New
      """
    And there are no unstaged changes
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames
      git add -- file.md
      """

  Scenario: precommit a fully staged file with spaces in the name
    Given a Git repository
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      """
    And a file "my file.md" with content
      """
      # Original
      """
    And I ran "git add -A"
    And I ran "git commit -m original"
    And I change file "my file.md" to
      """
      #     New
      """
    And I ran "git add -A"
    When executing "trident precommit --show=failed"
    Then it prints nothing to STDOUT
    And the staged changes are
      """
      diff --git a/my file.md b/my file.md
      index 6950014..e65f941 100644
      --- a/my file.md
      +++ b/my file.md
      @@ -1 +1 @@
      -# Original
      +# New
      """
    And there are no unstaged changes
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --no-renames
      git add -- 'my file.md'
      """
