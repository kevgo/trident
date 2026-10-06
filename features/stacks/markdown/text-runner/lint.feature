Feature: lint documentation with Text-Runner

  Background:
    Given a file "run-that-app" with content
      """
      biome 2.5.2
      delete-empty-folders 0.0.2
      node 26.4.0
      rumdl 0.2.14
      taplo 0.10.0
      text-runner 7.5.1
      """
    And a file "text-runner.jsonc" with content
      """
      {
      "format": "detailed",
      "systemTmp": true,
      "online": false
      }
      """
    And a file "rumdl.toml" with content
      """
      enable = []
      """

  Scenario: valid Markdown
    Given a file "one.md" with content
      """
      # One
      
      <a type="hello"></a>
      """
    And a file "text-runner/hello.js" with content
      """
      export default function hello(action) {
        action.log("Hello world!")
      }
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Markdown (rumdl)
      test Markdown (Text-Runner)
      one.md:3 -- Hello
      """
    And the exit code is 0
    And all files are unchanged

  Scenario: unformatted Markdown
    Given a file "one.md" with content
      """
      # One
      
      <a type="failing"></a>
      """
    And a file "text-runner/failing.js" with content
      """
      export default function failing() {
        throw new Error("Boom!");
      }
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      test Markdown (Text-Runner)
      one.md:3 -- Boom!
      """
    And the exit code is 1
    And all files are unchanged

  Scenario: invalid Markdown
    Given a file "main.md" with content
      """
      <a type="missing"></a>
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      test Markdown (Text-Runner)
      main.md:1 -- unknown action: missing
      """
    And the exit code is 1
    And all files are unchanged
