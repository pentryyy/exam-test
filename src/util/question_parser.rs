use std::fs;
use std::str::FromStr;
use anyhow::{anyhow, Context};
use crate::dto::question::Question;

pub fn parse_questions(path: &str) -> anyhow::Result<Vec<Question>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("не удалось прочитать файл {}", path))?;
    let content = content.replace("\r\n", "\n");
    let lines: Vec<&str> = content.split('\n').collect();

    let mut questions = Vec::new();
    let mut cur: Option<Question> = None;
    let mut mode = String::new();

    for (i, raw) in lines.iter().enumerate() {
        let line_no = i + 1;
        let line = raw.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("- question:") {
            if let Some(q) = cur.take() {
                questions.push(q);
            }
            let text = unquote(&line["- question:".len()..]);
            cur = Some(Question {
                text,
                options: Vec::new(),
                correct_indices: Vec::new(),
                correct_text: String::new(),
                accept: Vec::new(),
                line: line_no,
            });
            mode.clear();
            continue;
        }

        let cur_ref = cur.as_mut().ok_or_else(|| {
            anyhow!("строка {}: данные вне блока вопроса: {:?}", line_no, line)
        })?;

        if line == "options:" {
            mode = "options".to_string();
        } else if line == "accept:" {
            mode = "accept".to_string();
        } else if line.starts_with("- answer:") {
            let opt = unquote(&line["- answer:".len()..]);
            cur_ref.options.push(opt);
        } else if line.starts_with("correct:") {
            let val = line["correct:".len()..].trim();

            if val.starts_with('[') && val.ends_with(']') {
                let inner = &val[1..val.len() - 1];
                let indices: Vec<usize> = inner
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| usize::from_str(s))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| anyhow!("строка {}: ошибка парсинга индексов: {}", line_no, e))?;

                if indices.is_empty() {
                    return Err(anyhow!("строка {}: массив correct не может быть пустым", line_no));
                }
                cur_ref.correct_indices = indices;
            } else if let Ok(n) = usize::from_str(val) {
                cur_ref.correct_indices = vec![n];
            } else {
                cur_ref.correct_text = unquote(val);
            }
            mode.clear();
        } else if line.starts_with("- ") && mode == "accept" {
            let acc = unquote(&line["- ".len()..]);
            cur_ref.accept.push(acc);
        } else {
            return Err(anyhow!(
                "строка {}: не удалось разобрать: {:?}",
                line_no,
                line
            ));
        }
    }

    if let Some(q) = cur.take() {
        questions.push(q);
    }

    if questions.is_empty() {
        return Err(anyhow!("в файле {} не найдено ни одного вопроса", path));
    }

    for q in &questions {
        if q.text.is_empty() {
            return Err(anyhow!("строка {}: пустой текст вопроса", q.line));
        }
        if q.is_choice() {
            if q.correct_indices.is_empty() {
                return Err(anyhow!(
                    "строка {}: для вопроса с вариантами не указан correct индекс(ы)",
                    q.line
                ));
            }
            for &idx in &q.correct_indices {
                if idx >= q.options.len() {
                    return Err(anyhow!(
                        "строка {}: correct={} вне диапазона вариантов ({})",
                        q.line,
                        idx,
                        q.options.len()
                    ));
                }
            }
        } else if q.correct_text.is_empty() {
            return Err(anyhow!(
                "строка {}: у вопроса со свободным вводом не задан текстовый ответ",
                q.line
            ));
        }
    }

    Ok(questions)
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 {
        if s.starts_with('"') && s.ends_with('"') {
            let inner = &s[1..s.len() - 1];
            return inner
                .replace("\\\"", "\"")
                .replace("\\\\", "\\");
        }
        if s.starts_with('\'') && s.ends_with('\'') {
            let inner = &s[1..s.len() - 1];
            return inner.replace("''", "'");
        }
    }
    s.to_string()
}
