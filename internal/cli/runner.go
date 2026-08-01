package cli

import (
	"bufio"
	"exam-test/internal/config"
	"exam-test/internal/matcher"
	"exam-test/internal/model"
	"flag"
	"fmt"
	"math/rand"
	"os"
	"os/exec"
	"runtime"
	"strconv"

	"strings"
	"time"
)

func clearScreen() {
	if runtime.GOOS == "windows" {
		cmd := exec.Command("cmd", "/c", "cls")
		cmd.Stdout = os.Stdout

		// Игнорируем ошибку (если консоль не поддерживает).
		_ = cmd.Run()
	} else {
		fmt.Print("\033[H\033[2J")
	}
}

func isQuit(s string) bool {
	switch strings.ToLower(s) {
	case "!выход", "!quit", "!exit":
		return true
	}
	return false
}

func askLine(r *bufio.Reader) (string, bool) {
	line, err := r.ReadString('\n')
	if err != nil && line == "" {
		return "", true
	}
	line = strings.TrimRight(line, "\r\n")
	if isQuit(strings.TrimSpace(line)) {
		return "", true
	}
	return line, false
}

func isSkip(s string) bool {
	switch strings.ToLower(s) {
	case "!пропуск", "!skip", "":
		return true
	}
	return false
}

func waitForStart(r *bufio.Reader) bool {
	for {
		line, err := r.ReadString('\n')
		if err != nil {
			return false
		}
		line = strings.TrimSpace(line)
		if line == "" || line == "!старт" {
			return true
		}
		if isQuit(line) {
			return false
		}
		fmt.Println("Неизвестная команда. Введите !старт.")
	}
}

func askChoice(r *bufio.Reader, max int) (int, bool) {
	for {
		fmt.Print("\nВаш ответ (1-", max, "): ")
		line, stop := askLine(r)
		if stop {
			return 0, true
		}
		line = strings.TrimSpace(line)
		if isSkip(line) {
			return -1, false
		}
		num, err := strconv.Atoi(line)
		if err != nil || num < 1 || num > max {
			fmt.Printf("Введите число от 1 до %d.\n", max)
			continue
		}
		return num - 1, false
	}
}

func printHeader(pool, n int) {
	fmt.Println(strings.Repeat("=", 64))
	fmt.Println("ТЕСТ ЭКЗАМЕНА")
	fmt.Println(strings.Repeat("=", 64))
	fmt.Printf("Вопросов в базе: %d. В тесте: %d (выбраны случайно).\n", pool, n)
	fmt.Println("Для вопросов с вариантами введите номер варианта.")
	fmt.Println("Команды: !пропуск — пропустить вопрос, !выход — завершить тест досрочно, !старт - для начала теста.")
}

func progressBar(p float64) string {
	const width = 40
	filled := int(p / 100 * width)
	if filled > width {
		filled = width
	}
	if filled < 0 {
		filled = 0
	}
	return "[" + strings.Repeat("#", filled) + strings.Repeat(".", width-filled) + "]"
}

func printReport(cfg *config.Config, results []model.Result, planned int, aborted bool) {
	fmt.Println()
	fmt.Println(strings.Repeat("=", 64))
	fmt.Println("  РЕЗУЛЬТАТЫ")
	fmt.Println(strings.Repeat("=", 64))

	if len(results) == 0 {
		fmt.Println("Ни на один вопрос ответа не получено.")
		return
	}

	right := 0
	var wrong []model.Result
	for _, r := range results {
		if r.Correct {
			right++
		} else {
			wrong = append(wrong, r)
		}
	}

	// Процент считается от числа заданных вопросов.
	total := len(results)
	if aborted {
		fmt.Printf("Тест прерван: отвечено %d из %d вопросов.\n", total, planned)
	}
	percent := float64(right) / float64(total) * 100

	fmt.Printf("Правильных ответов: %d из %d\n", right, total)
	fmt.Printf("Результат: %.1f%% — %s\n", percent, cfg.Grade(percent))
	fmt.Println(progressBar(percent))

	if len(wrong) == 0 {
		fmt.Println("\nОшибок нет. Отличная работа!")
		return
	}

	fmt.Printf("\nНеверно отвеченные вопросы (%d):\n", len(wrong))
	for i, r := range wrong {
		fmt.Println(strings.Repeat("-", 64))
		fmt.Printf("%d. %s\n", i+1, r.Q.Text)
		fmt.Printf("   Ваш ответ:        %s\n", r.UserAnswer)
		fmt.Printf("   Правильный ответ: %s\n", r.Q.CorrectAnswerString())
	}
	fmt.Println(strings.Repeat("-", 64))
}

func runInteractive(cfg *config.Config, t model.Test, d time.Duration) error {
	rng := rand.New(rand.NewSource(time.Now().UnixNano()))
	n := t.Count
	if n > len(t.Questions) {
		n = len(t.Questions)
	}
	if n < 1 {
		return fmt.Errorf("количество вопросов должно быть больше нуля")
	}

	order := rng.Perm(len(t.Questions))
	selected := make([]model.Question, 0, n)
	for _, idx := range order[:n] {
		selected = append(selected, t.Questions[idx])
	}

	r := bufio.NewReader(os.Stdin)
	printHeader(len(t.Questions), n)

	if !waitForStart(r) {
		return nil
	}

	clearScreen()

	results := make([]model.Result, 0, n)
	aborted := false

	for i, q := range selected {
		if i > 0 {
			clearScreen()
		}

		fmt.Printf("\nВопрос %d из %d\n", i+1, n)
		fmt.Println(strings.Repeat("-", 64))
		fmt.Println(q.Text)

		var (
			userAnswer string
			ok         bool
			stop       bool
		)

		if q.IsChoice() {
			perm := rng.Perm(len(q.Options))
			shuffled := make([]string, len(q.Options))
			correctPos := -1
			for newPos, oldPos := range perm {
				shuffled[newPos] = q.Options[oldPos]
				if oldPos == q.CorrectIndex {
					correctPos = newPos
				}
			}
			fmt.Println()
			for j, opt := range shuffled {
				fmt.Printf("  %d) %s\n", j+1, opt)
			}
			choice, s := askChoice(r, len(shuffled))
			stop = s
			if !stop && choice >= 0 {
				userAnswer = shuffled[choice]
				ok = choice == correctPos
			}
		} else {
			fmt.Println("(введите ответ текстом)")
			line, s := askLine(r)
			stop = s
			if !stop {
				userAnswer = strings.TrimSpace(line)
				ok = matcher.MatchText(userAnswer, q)
			}
		}

		if stop {
			aborted = true
			break
		}

		if userAnswer == "" {
			userAnswer = "(нет ответа)"
		}
		if ok {
			fmt.Println("  Верно")
		} else {
			fmt.Println("  Неверно")
		}
		results = append(results, model.Result{Q: q, UserAnswer: userAnswer, Correct: ok})

		if i < n-1 {
			time.Sleep(d)
		}
	}

	printReport(cfg, results, n, aborted)

	fmt.Println("\nВведите !выход для выхода.")
	line, err := r.ReadString('\n')
	if err != nil && line == "!выход" {
		return err
	}

	return nil
}

func Run(cfg *config.Config) error {
	file := flag.String("file", "", "путь к YAML-файлу с вопросами (приоритет над test_path из конфига)")
	count := flag.Int("n", 0, "количество вопросов в тесте (приоритет над test_count из конфига)")
	flag.Parse()

	finalFile := cfg.TestPath
	if *file != "" {
		finalFile = *file
	}

	finalCount := cfg.TestCount
	if *count != 0 {
		finalCount = *count
	}

	delay, err := time.ParseDuration(cfg.ResultDelay)
	if err != nil || delay < 0 {
		return fmt.Errorf("некорректное значение result_delay=%q (ожидается положительное время, например 500ms)", cfg.ResultDelay)
	}

	pool, err := model.ParseQuestions(finalFile)
	if err != nil {
		return fmt.Errorf("ошибка чтения вопросов из файла %q: %w", finalFile, err)
	}

	return runInteractive(cfg, model.Test{
		Questions: pool,
		Count:     finalCount,
	}, delay)
}
