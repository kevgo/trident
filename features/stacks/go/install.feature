@online
Feature: install all Go tools

  Scenario: not installed
    Given a file "go.mod" with content
      """
      module example.com/demo
      go 1.21
      """
    And a file "main.go" with content
      """
      package main
      import "fmt"
      func main() {
      	fmt.Println(    "Hello, world!")
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines to STDERR
      """
      Talking to GitHub API (https://api.github.com/repos/mvdan/gofumpt/releases/latest) ... ok
      """
    And it prints the block
      """
      fix Go (gofumpt)
      main.go
      """
    And the exit code is 0
    And file "main.go" now has content
      """
      package main
      
      import "fmt"
      
      func main() {
      	fmt.Println("Hello, world!")
      }
      """
    And file "run-that-app" now has an additional line matching
      """
      delete-empty-folders \d+\.\d+\.\d+
      gofumpt \d+\.\d+\.\d+
      """
