use crate::dto::test::CfgTest;

#[derive(Debug, Clone)]
pub struct AnswerResult {
    pub question: CfgTest,
    pub user_answer: String,
    pub correct: bool,
}
