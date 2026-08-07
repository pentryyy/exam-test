#[derive(Debug, Clone)]
pub struct Question {
    pub text: String,
    pub options: Vec<String>,
    pub correct_index: Option<usize>,
    pub correct_text: String,
    pub accept: Vec<String>,
    pub line: usize,
}

impl Question {
    pub fn is_choice(&self) -> bool {
        !self.options.is_empty()
    }

    pub fn correct_answer_string(&self) -> String {
        if self.is_choice() {
            self.options[self.correct_index.unwrap()].clone()
        } else {
            self.correct_text.clone()
        }
    }
}
