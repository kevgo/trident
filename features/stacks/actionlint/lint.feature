Feature: lint GitHub Actions workflow files

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      actionlint 1.7.12
      delete-empty-folders 0.0.2
      """

  Scenario: valid workflow
    Given a file ".github/workflows/main.yml" with unix newlines
      """
      name: CI
      on:
        push:
          branches: [main]
      jobs:
        test:
          runs-on: ubuntu-latest
          steps:
            - uses: actions/checkout@v6
      """
    When executing "trident lint --show=output"
    Then it prints the lines to STDERR
      """
      1 YML, 1 other
      running 2 tasks
      """
    And it prints only these lines in any order
      """
      lint all (git diff HEAD --check)
      lint YML (Actionlint)
      """
    And the exit code is 0
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --untracked-files=all
      git status --porcelain=v1 -z --untracked-files=all
      git diff HEAD --check
      """

  Scenario: invalid workflow
    Given a file ".github/workflows/main.yml" with unix newlines
      """
      key: "
      """
    When executing "trident lint --show=output"
    Then it prints only these regexes in any order
      """
      lint all \(git diff HEAD --check\)
      lint YML \(Actionlint\)
      .github[/\\]workflows[/\\]main.yml:2:5: could not parse as YAML: found unexpected end of stream \[syntax-check\]
      """
    And the exit code is 1
    And all files are unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      git status --porcelain=v1 -z --untracked-files=all
      git status --porcelain=v1 -z --untracked-files=all
      git diff HEAD --check
      """
