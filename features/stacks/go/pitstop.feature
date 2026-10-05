Feature: pitstop Go

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
      golangci-lint 2.12.2
      """

  Scenario: unformatted Go
    Given a file "main.go" with content
      """
      package main
      import "fmt"
      func main() {
      fmt.Println("Hello")
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix Go (gofumpt)
      main.go
      lint Go (golangci-lint)
      0 issues.
      """
    And the exit code is 0
    And file "main.go" now has content
      """
      package main

      import "fmt"

      func main() {
      \tfmt.Println("Hello")
      }
      """

  Scenario: unformatted Go with lint errors
    Given a file "main.go" with content
      """
      package main
      import "fmt"
      func main() {
      	fmt.Printf(    "Hello")
      }
      """
    And a file "other.go" with content
      """
      package other
      import "fmt"
      func main() {
      fmt.Printf(    "Other")
      }
      """
    When executing "trident pitstop --show=output"
    Then it prints the lines
      """
      fix Go (gofumpt)
      main.go
      lint Go (golangci-lint)
      """
    And it prints the block
      """
      found packages main (main.go) and other (other.go)
      """
    And it prints the lines
      """
      other.go
      """
    And the exit code is failure
    And file "main.go" now has content
      """
      package main

      import "fmt"

      func main() {
      \tfmt.Printf("Hello")
      }
      """
    And file "other.go" now has content
      """
      package other

      import "fmt"

      func main() {
      \tfmt.Printf("Other")
      }
      """
