package main

import (
	"exam-test/internal/cli"
	"fmt"
	"os"
)

func main() {
	if err := cli.Run(); err != nil {
		_, err := fmt.Fprintf(os.Stderr, "Ошибка: %v\n", err)
		if err != nil {
			return
		}
		os.Exit(1)
	}
}
