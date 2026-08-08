use crate::types::answer_type::{AnswerKind, AnswerType};

#[derive(Debug, Clone)]
pub struct Question {
    pub text: String,
    pub options: Vec<String>,
    pub correct: AnswerType,
    pub accept: Vec<String>,
    pub line: usize,
}

impl Question {
    fn log_answer_type(&self) {
        match &self.correct {
            AnswerType::SingleAnswer(_) => println!("[ВЫБЕРИТЕ ОДИН ОТВЕТ]"),
            AnswerType::MultipleAnswer(_) => println!("[ВЫБЕРИТЕ НЕСКОЛЬКО ОТВЕТОВ]"),
            AnswerType::TextAnswer(_) => println!("[ВВЕДИТЕ ТЕКСТОВЫЙ ОТВЕТ]"),
        }
    }

    pub fn get_answer_type(&self) -> AnswerKind {
        self.log_answer_type();
        self.correct.kind()
    }

    pub fn is_choice(&self) -> bool {
        !self.options.is_empty()
    }

    pub fn correct_answer_string(&self) -> String {
        match &self.correct {
            AnswerType::SingleAnswer(idx) => self.options[*idx].clone(),
            AnswerType::MultipleAnswer(indices) => indices
                .iter()
                .map(|&i| self.options[i].clone())
                .collect::<Vec<_>>()
                .join(", "),
            AnswerType::TextAnswer(text) => text.clone(),
        }
    }
}
