Feature: fix Go

  Background:
    Given a file "go.mod" with content
      """
      module example.com/demo
      go 1.21
      """
    And a file "run-that-app" with content
      """
      delete-empty-folders 0.0.2
      gofumpt 0.10.0
      golangci-lint 2.15.0
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
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Go (gofumpt)
      """
    And the exit code is 0
    And file "main.go" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: unformatted Go
    Given a file "main.go" with content
      """
      package main
      import "fmt"
      func main() {
      	fmt.Println(    "Hello, world!")
      }
      """
    And a file "other.go" with content
      """
      package other
      import "fmt"
      func main() {
      	fmt.Println(    "Hello, other!")
      }
      """
    When executing "trident fix --show=output"
    Then it prints the lines
      """
      fix Go (gofumpt)
      main.go
      """
    And it prints the lines
      """
      other.go
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
    And file "other.go" now has content
      """
      package other

      import "fmt"

      func main() {
      	fmt.Println("Hello, other!")
      }
      """
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """

  Scenario: invalid Go
    Given a file "main.go" with content
      """
      package main
      import "fmt"
      func main() {
      	fmt.Println("
      }
      """
    When executing "trident fix --show=output"
    Then it prints the block
      """
      fix Go (gofumpt)
      main.go:4:14: string literal not terminated
      """
    And the exit code is 2
    And file "main.go" is unchanged
    And it runs the Git commands
      """
      git rev-parse --is-inside-work-tree
      """
