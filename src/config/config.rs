use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::fs;
use std::time::Duration;

pub const CONFIG_PATH: &str = "config/config.yaml";

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    humantime::parse_duration(&s)
        .map_err(|e| serde::de::Error::custom(format!("некорректное значение result_delay={:?}: {}", s, e)))
}

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
    #[serde(deserialize_with = "deserialize_duration")]
    pub result_delay: Duration,
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
