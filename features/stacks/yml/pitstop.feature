Feature: pitstop YML

  Scenario: unformatted YML
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      prettier 3.7.0
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
      fix YML (Prettier)
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

  Scenario: unformatted YML with lint errors
    Given a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      node 26.4.0
      prettier 3.7.0
      """
    And a file "main.yml" with content
      """
      key:     value
      key:     other
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix YML (Prettier)
      [error] main.yml: SyntaxError: Map keys must be unique; "key" is repeated (1:1)
      """
    And the exit code is 2
    And file "main.yml" now has content
      """
      key:     value
      key:     other
      """
