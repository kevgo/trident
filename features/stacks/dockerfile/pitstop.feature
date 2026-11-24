Feature: pitstop Dockerfile with hadolint

  Background:
    Given a file "run-that-app" with unix newlines
      """
      delete-empty-folders 0.0.2
      hadolint 2.15.1
      """

  Scenario: valid Dockerfile
    Given a file "Dockerfile" with unix newlines
      """
      FROM alpine:3.20
      USER 65534
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      lint Dockerfile (hadolint)
      """
    And the exit code is 0
    And file "Dockerfile" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid Dockerfile
    Given a file "Dockerfile" with unix newlines
      """
      FROM alpine:3.20
      WORKDIR tmp
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      lint Dockerfile (hadolint)
      Dockerfile:2 DL3000 error: Use absolute WORKDIR
      """
    And the exit code is 1
    And file "Dockerfile" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
