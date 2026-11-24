Feature: disable an application's lint operation

  Background:
    Given a file "run-that-app" with unix newlines
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      rumdl 0.2.14
      taplo 0.10.0
      """
    And a file "trident.json" with unix newlines
      """
      {
        "applications": {
          "taplo": {
            "operations": {
              "lint": {
                "enabled": false
              }
            }
          }
        }
      }
      """
    And a file "other.md" with unix newlines
      """
      # a markdown file
      """
    And a file "Cargo.toml" with unix newlines
      """
      [package]
      name =      "demo"

      [lints.clippy]
      pedantic = { level = "warn" }
      """

  Scenario: lint skips the application
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Markdown (rumdl)
      """
    And it does not print
      """
      Taplo
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: fix still runs the application
    When executing "trident fix --show=verbose"
    Then it prints the lines matching
      """
      fix TOML \(Taplo\)
      taplo(.exe)? format Cargo.toml
      """
    And it prints the lines matching
      """
      fix Markdown \(rumdl\)
      rumdl(.exe)? fmt other.md
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: fix-unsafe still runs the application
    When executing "trident fix-unsafe --show=verbose"
    Then it prints the lines matching
      """
      unsafe-fix TOML \(Taplo\)
      taplo(.exe)? format --force Cargo.toml
      """
    And the exit code is 0
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
