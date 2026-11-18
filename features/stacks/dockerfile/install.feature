@online
Feature: install hadolint

  Scenario: not installed
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      """
    And a file "Dockerfile" with content
      """
      FROM alpine:3.20
      USER 65534
      """
    When executing "trident lint --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/hadolint/hadolint/releases/latest) ... ok
      """
    And it prints the lines
      """
      lint Dockerfile (hadolint)
      """
    And the exit code is 0
    And file "run-that-app" now has an additional line matching
      """
      hadolint \d+\.\d+\.\d+
      """
