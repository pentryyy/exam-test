use crate::types::answer_type::AnswerType;

#[derive(Debug, Clone)]
pub struct Question {
    pub text: String,
    pub options: Vec<String>,
    pub correct_indices: Vec<usize>,
    pub correct_text: String,
    pub accept: Vec<String>,
    pub line: usize,
}

impl Question {
    pub fn is_choice(&self) -> bool {
        !self.options.is_empty()
    }

    fn is_multiple_choice(&self) -> bool {
        self.is_choice() && self.correct_indices.len() > 1
    }

    pub fn get_answer_type(&self) -> AnswerType {
        if self.is_choice() {
            if self.is_multiple_choice() {
                println!("[ВЫБЕРИТЕ НЕСКОЛЬКО ОТВЕТОВ]");
                AnswerType::MultipleAnswer
            } else {
                println!("[ВЫБЕРИТЕ ОДИН ОТВЕТ]");
                AnswerType::SingleAnswer
            }
        } else {
            println!("[ВВЕДИТЕ ТЕКСТОВЫЙ ОТВЕТ]");
            AnswerType::TextAnswer
        }
    }

    pub fn correct_answer_string(&self) -> String {
        if self.is_choice() {
            let answers: Vec<String> = self.correct_indices
                .iter()
                .map(|&idx| self.options[idx].clone())
                .collect();
            answers.join(", ")
        } else {
            self.correct_text.clone()
        }
    }
}
