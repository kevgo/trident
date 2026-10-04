Feature: disable an application's fix operation

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      prettier 3.7.0
      rumdl 0.2.14
      taplo 0.10.0
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "taplo": {
            "operations": {
              "fix": {
                "enabled": false
              }
            }
          }
        }
      }
      """
    And a file "other.md" with content
      """
      # a markdown file
      """
    And a file "Cargo.toml" with content
      """
      [package]
      name =      "demo"
      
      [lints.clippy]
      pedantic = { level = "warn" }
      """

  Scenario: fix skips the application
    When executing "trident fix --show=output"
    Then it prints the block
      """
      fix Markdown (rumdl)
      """
    And it does not print
      """
      Taplo
      """
    And file "Cargo.toml" is unchanged
    And the exit code is 0

  Scenario: lint still runs the application
    When executing "trident lint --show=verbose"
    Then it prints the lines matching
      """
      lint TOML \(Taplo\)
      taplo(.exe)? lint Cargo.toml
      """
    And it prints the lines matching
      """
      lint Markdown \(rumdl\)
      rumdl(.exe)? check other.md
      """
    And the exit code is 0

  Scenario: fix-unsafe still runs the application
    When executing "trident fix-unsafe --show=verbose"
    Then it prints the lines matching
      """
      unsafe-fix TOML \(Taplo\)
      taplo(.exe)? format --force Cargo.toml
      """
    And the exit code is 0
