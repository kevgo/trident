Feature: "trident postedit" skips disabled applications

  Scenario: skips a disabled application
    Given a file "run-that-app" with content
      """
      biome 2.4.0
      delete-empty-folders 0.0.2
      taplo 0.10.0
      """
    And a file "config.toml" with content
      """
      key =     "value"
      """
    And a file "trident.json" with content
      """
      {
        "applications": {
          "taplo": {
            "enabled": false
          }
        }
      }
      """
    When executing "trident postedit --show=output"
    Then it does not print
      """
      Taplo
      """
    And file "config.toml" is unchanged
    And the exit code is 0
