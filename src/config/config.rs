use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

pub const CONFIG_PATH: &str = "config/config.yaml";

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

        cfg.validate()?;

        cfg.grades
            .sort_by(|a, b| b.threshold.partial_cmp(&a.threshold).unwrap());

        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        if self.grades.is_empty() {
            return Err(anyhow!("в конфиге не задано ни одного порога оценки"));
        }

        for (i, grade) in self.grades.iter().enumerate() {
            if grade.label.is_empty() {
                return Err(anyhow!(
                    "порог #{}: не задана метка (label)",
                    i
                ));
            }
            if !(0.0..=100.0).contains(&grade.threshold) {
                return Err(anyhow!(
                    "порог #{} ({:?}): значение {} вне диапазона [0, 100]",
                    i,
                    grade.label,
                    grade.threshold
                ));
            }
        }

        if self.test_path.is_empty() {
            return Err(anyhow!("поле test_path не задано"));
        }

        if self.test_count <= 0 {
            return Err(anyhow!(
                "поле test_count должно быть больше нуля (текущее {})",
                self.test_count
            ));
        }

        if self.result_delay.is_empty() {
            return Err(anyhow!("поле result_delay не задано"));
        }

        humantime::parse_duration(&self.result_delay)
            .map_err(|_| anyhow!(
                "некорректное значение result_delay={:?} (ожидается время, например 500ms)",
                self.result_delay
            ))?;

        Ok(())
    }

    pub fn grade(&self, p: f64) -> String {
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
