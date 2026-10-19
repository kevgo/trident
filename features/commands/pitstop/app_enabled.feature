Feature: "trident pitstop" skips disabled applications

  Background:
    Given a file "run-that-app" with content
      """
      biome 0.21.0
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """
    And a file "config.toml" with content
      """
      key =     "value"
      """

  Scenario: skips a disabled application
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "taplo": {
            "enabled": false
          }
        }
      }
      """
    When executing "trident pitstop --show=output"
    Then it does not print
      """
      Taplo
      """
    And it prints the block
      """
      delete empty folders
      """
    And file "config.toml" is unchanged
    And the exit code is 0

  Scenario: skips a disabled global application
    Given a file "trident.json" with content
      """
      {
        "applications": {
          "delete_empty_folders": {
            "enabled": false
          }
        }
      }
      """
    When executing "trident pitstop --show=output"
    Then it does not print
      """
      delete empty folders
      """
    And it prints the block
      """
      Taplo
      """
    And file "config.toml" now has content
      """
      key = "value"
      """
    And the exit code is 0
