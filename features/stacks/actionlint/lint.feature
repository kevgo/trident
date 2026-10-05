Feature: lint GitHub Actions workflow files

  Background:
    Given a Git repository
    And a file "run-that-app" with content
      """
      actionlint 1.7.12
      delete-empty-folders 0.0.2
      """

  Scenario: valid workflow
    Given a file ".github/workflows/main.yml" with content
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
    Then it prints only these lines in any order
      """
      lint Git diff markers (git diff HEAD --check)
      GitHub Actions (actionlint)
      """
    And it prints the lines to STDERR
      """
      1 YML, 1 other
      running 2 tools
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: invalid workflow
    Given a file ".github/workflows/main.yml" with content
      """
      key: "
      """
    When executing "trident lint --show=output"
    Then it prints only these regexes in any order
      """
      lint Git diff markers \(git diff HEAD --check\)
      GitHub Actions \(actionlint\)
      .github[/\\]workflows[/\\]main.yml:2:5: could not parse as YAML: found unexpected end of stream \[syntax-check\]
      """
    And the exit code is 1
    And all files are unchanged
