package main

import (
	"exam-test/internal/cli"
	"exam-test/internal/config"
	"os"
)

func main() {
	cfg, err := config.Load()
	if err != nil {
		cli.LogError("Ошибка загрузки конфига", err)
		os.Exit(1)
	}

	if err := cli.Run(cfg); err != nil {
		cli.LogError("Ошибка запуска тестов", err)
		os.Exit(1)
	}
}
