use crate::dto::question::Question;

#[derive(Debug, Clone)]
pub struct Test {
    pub questions: Vec<Question>,
    pub count: usize,
}
