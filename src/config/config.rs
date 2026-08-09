use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

pub const CONFIG_PATH: &str = "config/config.yaml";

pub trait Grader {
    fn grade(&self, percent: f64) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeThreshold {
    pub threshold: f64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub grades: Vec<GradeThreshold>,
    pub test_path: String,
    pub test_count: usize,
    pub result_delay: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let data = fs::read_to_string(CONFIG_PATH)
            .with_context(|| format!("не удалось прочитать конфиг {:?}", CONFIG_PATH))?;

        let mut cfg: Config = serde_yaml::from_str(&data)
            .with_context(|| format!("ошибка парсинга конфига {:?}", CONFIG_PATH))?;

        cfg.grades
            .sort_by(|a, b| b.threshold.partial_cmp(&a.threshold).unwrap());

        Ok(cfg)
    }
}

impl Grader for Config {
    fn grade(&self, p: f64) -> String {
        if self.grades.is_empty() {
            return "ошибка: конфиг пуст".to_string();
        }

        for grade in &self.grades {
            if p >= grade.threshold {
                return grade.label.clone();
            }
        }

        self.grades.last().unwrap().label.clone()
    }
}
