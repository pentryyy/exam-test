package parser

import (
	"exam-test/internal/model"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func ParseQuestions(path string) ([]model.Question, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var (
		questions []model.Question
		cur       *model.Question
		mode      string
	)

	flush := func() {
		if cur != nil {
			questions = append(questions, *cur)
			cur = nil
		}
	}

	lines := strings.Split(strings.ReplaceAll(string(data), "\r\n", "\n"), "\n")
	for i, raw := range lines {
		lineNo := i + 1
		line := strings.TrimSpace(raw)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		switch {
		case strings.HasPrefix(line, "- question:"):
			flush()
			cur = &model.Question{
				Text:         unquote(strings.TrimPrefix(line, "- question:")),
				CorrectIndex: -1,
				Line:         lineNo,
			}
			mode = ""
		case cur == nil:
			return nil, fmt.Errorf("строка %d: данные вне блока вопроса: %q", lineNo, line)
		case strings.HasPrefix(line, "options:"):
			mode = "options"
		case strings.HasPrefix(line, "accept:"):
			mode = "accept"
		case strings.HasPrefix(line, "- answer:"):
			cur.Options = append(cur.Options, unquote(strings.TrimPrefix(line, "- answer:")))
		case strings.HasPrefix(line, "correct:"):
			val := strings.TrimSpace(strings.TrimPrefix(line, "correct:"))
			if n, err := strconv.Atoi(val); err == nil {
				cur.CorrectIndex = n
			} else {
				cur.CorrectText = unquote(val)
			}
			mode = ""
		case strings.HasPrefix(line, "- ") && mode == "accept":
			cur.Accept = append(cur.Accept, unquote(strings.TrimPrefix(line, "- ")))
		default:
			return nil, fmt.Errorf("строка %d: не удалось разобрать: %q", lineNo, line)
		}
	}
	flush()

	if len(questions) == 0 {
		return nil, fmt.Errorf("в файле %s не найдено ни одного вопроса", path)
	}

	for _, q := range questions {
		if q.Text == "" {
			return nil, fmt.Errorf("строка %d: пустой текст вопроса", q.Line)
		}
		if q.IsChoice() {
			if q.CorrectIndex < 0 || q.CorrectIndex >= len(q.Options) {
				return nil, fmt.Errorf("строка %d: correct=%d вне диапазона вариантов (%d)",
					q.Line, q.CorrectIndex, len(q.Options))
			}
		} else if q.CorrectText == "" {
			return nil, fmt.Errorf("строка %d: у вопроса со свободным вводом не задан текстовый ответ", q.Line)
		}
	}
	return questions, nil
}

func unquote(s string) string {
	s = strings.TrimSpace(s)
	if len(s) >= 2 {
		if s[0] == '"' && s[len(s)-1] == '"' {
			s = s[1 : len(s)-1]
			s = strings.ReplaceAll(s, `\"`, `"`)
			s = strings.ReplaceAll(s, `\\`, `\`)
			return s
		}
		if s[0] == '\'' && s[len(s)-1] == '\'' {
			s = s[1 : len(s)-1]
			return strings.ReplaceAll(s, "''", "'")
		}
	}
	return s
}
