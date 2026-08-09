use crate::types::answer_type::{AnswerKind, AnswerType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfgOption {
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfgTest {
    pub question: String,
    #[serde(default)]
    pub options: Vec<CfgOption>,
    pub correct: AnswerType,
    #[serde(default)]
    pub accept: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfgTests {
    pub questions: Vec<CfgTest>,
}

#[derive(Debug, Clone)]
pub struct CliTests {
    pub questions: Vec<CfgTest>,
    pub count: usize,
}

impl CfgTest {
    fn log_answer_type(&self) {
        match &self.correct {
            AnswerType::SingleAnswer(_) => println!("[ВЫБЕРИТЕ ОДИН ОТВЕТ]"),
            AnswerType::MultipleAnswer(_) => println!("[ВЫБЕРИТЕ НЕСКОЛЬКО ОТВЕТОВ]"),
            AnswerType::TextAnswer(_) => println!("[ВВЕДИТЕ ТЕКСТОВЫЙ ОТВЕТ]"),
        }
    }

    pub fn get_answer_type(&self) -> AnswerKind {
        self.log_answer_type();
        self.correct.kind()
    }

    pub fn correct_answer_string(&self) -> String {
        match &self.correct {
            AnswerType::SingleAnswer(idx) => self.options[*idx].answer.clone(),
            AnswerType::MultipleAnswer(indices) => indices
                .iter()
                .map(|&i| self.options[i].answer.clone())
                .collect::<Vec<String>>()
                .join(", "),
            AnswerType::TextAnswer(text) => text.clone(),
        }
    }
}

impl CfgTests {
    pub fn load(path: &str) -> Result<Self> {
        let data = fs::read_to_string(path)
            .with_context(|| format!("не удалось прочитать файл {:?}", path))?;

        let cfg_tests: CfgTests = serde_yaml::from_str(&data)
            .with_context(|| format!("ошибка парсинга файла {:?}", path))?;

        Ok(cfg_tests)
    }
}
