Feature: pitstop YML

  Scenario: unformatted YML
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      yamlfmt 0.21.0
      """
    And a file "main.yml" with content
      """
      key:     value
      """
    And a file "other.yml" with content
      """
      key:     other
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix YML (yamlfmt)
      """
    And the exit code is 0
    And file "main.yml" now has content
      """
      key: value
      """
    And file "other.yml" now has content
      """
      key: other
      """

  Scenario: duplicate keys
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      yamlfmt 0.21.0
      """
    And a file "main.yml" with content
      """
      key:     value
      key:     other
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix YML (yamlfmt)
      """
    And the exit code is 0
    And file "main.yml" now has content
      """
      key: value
      key: other
      """
