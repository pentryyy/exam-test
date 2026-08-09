use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum AnswerKind {
    SingleAnswer,
    MultipleAnswer,
    TextAnswer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnswerType {
    SingleAnswer(usize),
    MultipleAnswer(HashSet<usize>),
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
