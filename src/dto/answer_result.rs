use crate::dto::question::Question;

#[derive(Debug, Clone)]
pub struct AnswerResult {
    pub question: Question,
    pub user_answer: String,
    pub correct: bool,
}