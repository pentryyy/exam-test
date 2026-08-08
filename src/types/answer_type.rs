#[derive(Debug, Clone, PartialEq)]
pub enum AnswerKind {
    SingleAnswer,
    MultipleAnswer,
    TextAnswer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnswerType {
    SingleAnswer(usize),
    MultipleAnswer(Vec<usize>),
    TextAnswer(String),
}

impl AnswerType {
    pub fn kind(&self) -> AnswerKind {
        match self {
            AnswerType::SingleAnswer(_) => AnswerKind::SingleAnswer,
            AnswerType::MultipleAnswer(_) => AnswerKind::MultipleAnswer,
            AnswerType::TextAnswer(_) => AnswerKind::TextAnswer,
        }
    }
}
