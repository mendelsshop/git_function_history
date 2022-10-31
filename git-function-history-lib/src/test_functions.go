package main

import (
	"fmt"
	"os"
)

func main() {
	empty_test("1", 2, "3")
	fmt.Println("Hello World!")
}
// doc comment

func empty_test(c, a int, b string) {
	fmt.Println("Hello World!")
}