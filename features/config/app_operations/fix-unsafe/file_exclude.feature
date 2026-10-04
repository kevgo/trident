Feature: exclude a file from being unsafe-fixed by a specific app only

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      prettier 3.7.0
      taplo 0.10.0
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "taplo": {
            "operations": {
              "fix-unsafe": {
                "ignore-files": ["Cargo.toml"]
              }
            }
          }
        }
      }
      """
    And a file "other.toml" with content
      """
      key =     "value"
      """
    And a file "Cargo.toml" with content
      """
      [package]
      name =      "demo"
      
      [lints.clippy]
      pedantic = { level = "warn" }
      """

  Scenario: fix-unsafe ignores the file
    When executing "trident fix-unsafe --show=verbose"
    Then it prints the lines matching
      """
      unsafe-fix TOML \(Taplo\)
      taplo(.exe)? format --force other.toml
      """
    And it does not print
      """
      Cargo.toml
      """
    And file "other.toml" now has content
      """
      key = "value"
      """
    And file "Cargo.toml" is unchanged
    And the exit code is 0

  Scenario: lint still lints the file
    When executing "trident lint --show=verbose"
    Then it prints the lines matching
      """
      lint TOML \(Taplo\)
      taplo(.exe)? lint Cargo.toml other.toml
      """
    And the exit code is 0

  Scenario: fix still formats the file
    When executing "trident fix --show=verbose"
    Then it prints the lines matching
      """
      fix TOML \(Taplo\)
      taplo(.exe)? format Cargo.toml other.toml
      """
    And file "other.toml" now has content
      """
      key = "value"
      """
    And file "Cargo.toml" now has content
      """
      [package]
      name = "demo"
      
      [lints.clippy]
      pedantic = { level = "warn" }
      """
    And the exit code is 0
