use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum AnswerKind {
    Single,
    Multiple,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnswerType {
    Single(usize),
    Multiple(HashSet<usize>),
    Text(String),
}

impl AnswerType {
    pub fn kind(&self) -> AnswerKind {
        match self {
            AnswerType::Single(_) => AnswerKind::Single,
            AnswerType::Multiple(_) => AnswerKind::Multiple,
            AnswerType::Text(_) => AnswerKind::Text,
        }
    }
}
