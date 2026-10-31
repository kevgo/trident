Feature: GitHub Actions workflow files not in a Git repository

  Scenario: valid workflow
    Given a file "run-that-app" with content
      """
      actionlint 1.7.12
      delete-empty-folders 0.0.2
      """
    And a file ".github/workflows/main.yml" with content
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
    Then it prints the block matching
      """
      lint YML \(Actionlint\)
      no project was found in any parent directories of ".*". check workflows directory is put correctly in your Git repository
      """
    And it prints the lines to STDERR
      """
      1 YML, 1 other
      running 1 tools
      """
    And the exit code is 3
    And all files are unchanged
