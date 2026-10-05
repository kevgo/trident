Feature: lint Go

  Background:
    Given a file "go.mod" with content
      """
      module example.com/demo
      go 1.21
      """
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      golangci-lint 2.12.2
      """

  Scenario: valid Go
    Given a file "main.go" with content
      """
      package main
      import "fmt"
      func main() {
      	fmt.Println("Hello, world!")
      }
      """
    When executing "trident lint --show=output"
    Then it prints the lines
      """
      lint Go (golangci-lint)
      """
    And the exit code is 0
    And file "main.go" is unchanged

  Scenario: invalid Go
    Given a file "main.go" with content
      """
      package main
      import "fmt"
      func main() {
      	fmt.Println("
      }
      """
    When executing "trident lint --show=output"
    Then it prints the block
      """
      lint Go (golangci-lint)
      """
    And it prints the block
      """
      string literal not terminated
      """
    And the exit code is failure
    And file "main.go" is unchanged
