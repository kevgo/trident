Feature: ignore files for an application

  Background:
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      prettier 3.7.0
      taplo 0.10.0
      """
    And a file "Cargo.toml" with content
      """
      [package]
      name = "demo"
      """
    And a file "config.toml" with content
      """
      key = "value"
      """

@this
  Scenario: lint honors application-level ignored files
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "taplo": {
            "ignore-files": ["Cargo.toml"]
          }
        }
      }
      """
    When executing "trident lint --show=verbose"
    Then it prints the block matching
      """
      lint TOML \(Taplo\)
      \S+[/\\]taplo(\.exe)? lint config\.toml
      """
    And it does not print
      """
      Cargo.toml
      """
    And the exit code is 0

  Scenario: fix honors application-level ignored files
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "taplo": {
            "ignore-files": ["Cargo.toml"]
          }
        }
      }
      """
    And a file "config.toml" with content
      """
      key =     "value"
      """
    When executing "trident fix --show=verbose"
    Then it prints the block matching
      """
      fix TOML \(Taplo\)
      \S+/taplo format config\.toml\n
      """
    And it does not print
      """
      Cargo.toml
      """
    And file "config.toml" now has content
      """
      key = "value"
      """
    And file "Cargo.toml" is unchanged
    And the exit code is 0

  Scenario: lint honors globally ignored files
    Given a file "trident.json" with content
      """
      {
        "ignore-files": ["Cargo.toml"]
      }
      """
    When executing "trident lint --show=verbose"
    Then it prints the block matching
      """
      lint TOML \(Taplo\)
      \S+/taplo lint config\.toml\n
      """
    And it does not print
      """
      Cargo.toml
      """
    And the exit code is 0

  Scenario: fix honors globally ignored files
    Given a file "trident.json" with content
      """
      {
        "ignore-files": ["Cargo.toml"]
      }
      """
    When executing "trident fix --show=verbose"
    Then it prints the block matching
      """
      fix TOML \(Taplo\)
      \S+/taplo format config\.toml\n
      """
    And it does not print
      """
      Cargo.toml
      """
    And the exit code is 0
