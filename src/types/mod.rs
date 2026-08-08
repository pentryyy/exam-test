pub mod answer_type;

#[derive(Debug, Clone, PartialEq)]
pub enum AnswerType {
    Single,
    Multiple,
    Text,
}
