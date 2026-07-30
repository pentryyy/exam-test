package model

type Question struct {
	Text         string
	Options      []string
	CorrectIndex int
	CorrectText  string
	Accept       []string
	Line         int
}

func (q Question) IsChoice() bool {
	return len(q.Options) > 0
}

func (q Question) CorrectAnswerString() string {
	if q.IsChoice() {
		return q.Options[q.CorrectIndex]
	}
	return q.CorrectText
}

type Result struct {
	Q          Question
	UserAnswer string
	Correct    bool
}
