use crate::dto::test::CfgTest;

#[derive(Debug, Clone)]
pub struct AnswerResult {
    pub question: CfgTest,
    pub user_answer: String,
    pub correct: bool,
}

impl AnswerResult {
    pub fn log_answer(&self) {
        let pad = "  ";
        if self.correct {
            println!("{}Верно", pad);
        } else {
            println!("{}Неверно", pad);
        }
    }
}
