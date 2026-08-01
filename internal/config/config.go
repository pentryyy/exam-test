package config

import (
	"fmt"
	"os"
	"sort"
	"time"

	"gopkg.in/yaml.v3"
)

const ConfigPath = "config/config.yaml"

type GradeThreshold struct {
	Threshold float64 `yaml:"threshold"`
	Label     string  `yaml:"label"`
}

type Config struct {
	Grades      []GradeThreshold `yaml:"grades"`
	TestPath    string           `yaml:"test_path"`
	TestCount   int              `yaml:"test_count"`
	ResultDelay string           `yaml:"result_delay"`
}

func Load() (*Config, error) {
	data, err := os.ReadFile(ConfigPath)
	if err != nil {
		return nil, fmt.Errorf("не удалось прочитать конфиг %q: %w", ConfigPath, err)
	}

	var cfg Config
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("ошибка парсинга конфига %q: %w", ConfigPath, err)
	}

	if err := cfg.validate(); err != nil {
		return nil, err
	}

	// Сортируем по убыванию порога, чтобы grade() работал корректно
	sort.Slice(cfg.Grades, func(i, j int) bool {
		return cfg.Grades[i].Threshold > cfg.Grades[j].Threshold
	})

	return &cfg, nil
}

func (c *Config) validate() error {
	if len(c.Grades) == 0 {
		return fmt.Errorf("в конфиге не задано ни одного порога оценки")
	}
	for i, g := range c.Grades {
		if g.Label == "" {
			return fmt.Errorf("порог #%d: не задана метка (label)", i)
		}
		if g.Threshold < 0 || g.Threshold > 100 {
			return fmt.Errorf("порог #%d (%q): значение %v вне диапазона [0, 100]", i, g.Label, g.Threshold)
		}
	}

	if c.TestPath == "" {
		return fmt.Errorf("поле test_path не задано")
	}
	if c.TestCount <= 0 {
		return fmt.Errorf("поле test_count должно быть больше нуля (текущее %d)", c.TestCount)
	}
	if c.ResultDelay == "" {
		return fmt.Errorf("поле result_delay не задано")
	}
	if _, err := time.ParseDuration(c.ResultDelay); err != nil {
		return fmt.Errorf("некорректное значение result_delay=%q (ожидается время, например 500ms)", c.ResultDelay)
	}
	return nil
}

func (c *Config) Grade(p float64) string {
	if len(c.Grades) == 0 {
		return "ошибка: конфиг пуст"
	}

	for _, g := range c.Grades {
		if p >= g.Threshold {
			return g.Label
		}
	}

	// Если передали отрицательное число, возвращаем самый нижний порог из YAML
	return c.Grades[len(c.Grades)-1].Label
}
