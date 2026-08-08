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

    pub fn is_multiple_choice(&self) -> bool {
        self.is_choice() && self.correct_indices.len() > 1
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
