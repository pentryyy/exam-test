package main

import (
	"exam-test/internal/cli"
	"os"
)

func main() {
	if err := cli.Run(); err != nil {
		cli.LogError("Ошибка: переменная окружения TEST_PATH не задана")
		os.Exit(1)
	}
}
