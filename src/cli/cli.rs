use anyhow::{bail, Context, Result};
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;
use rand::prelude::SliceRandom;
use crate::config::config::Config;
use crate::dto::answer_result::AnswerResult;
use crate::dto::question::Question;
use crate::dto::test::Test;
use crate::util::question_parser::parse_questions;
use crate::util::text_matcher::match_text;

fn clear_screen() -> Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "cls"]).status()?;
        Ok(())
    } else {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush()?;
        Ok(())
    }
}

fn ask_line() -> Result<(String, bool)> {
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Ok((String::new(), true));
    }
    let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
    if is_quit(trimmed) {
        return Ok((String::new(), true));
    }
    Ok((trimmed.to_string(), false))
}

fn is_skip(s: &str) -> bool {
    let low = s.to_lowercase();
    low == "!пропуск" || low == "!skip"
}

fn is_restart(s: &str) -> bool {
    let low = s.to_lowercase();
    low == "!рестарт" || low == "!restart"
}

fn is_quit(s: &str) -> bool {
    let low = s.to_lowercase();
    low == "!выход" || low == "!quit" || low == "!exit"
}

fn wait_for_start() -> Result<bool> {
    loop {
        let (line, stop) = ask_line()?;
        if stop {
            return Ok(false);
        }
        let line = line.trim();
        if line.is_empty() || line == "!старт" {
            return Ok(true);
        }
        if is_quit(line) {
            return Ok(false);
        }
        println!("Неизвестная команда. Введите !старт.");
    }
}

fn ask_multiple_choice(max: usize) -> Result<(Vec<usize>, bool)> {
    loop {
        print!("Ваши ответы: ");
        let (line, stop) = ask_line()?;
        if stop {
            return Ok((Vec::new(), true));
        }
        let line = line.trim();
        if is_skip(line) {
            return Ok((Vec::new(), false));
        }

        if let Ok(num) = line.parse::<usize>() {
            if num >= 1 && num <= max {
                return Ok((vec![num - 1], false));
            }
        }

        let mut indices = Vec::new();
        let parts: Vec<&str> = line.split(',').collect();
        let mut valid = true;

        for part in parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(num) = trimmed.parse::<usize>() {
                if num >= 1 && num <= max {
                    if !indices.contains(&(num - 1)) {
                        indices.push(num - 1);
                    }
                } else {
                    valid = false;
                    break;
                }
            } else {
                valid = false;
                break;
            }
        }

        if valid && !indices.is_empty() {
            indices.sort();
            return Ok((indices, false));
        }
        println!("Введите корректные номера от 1 до {} через запятую.", max);
    }
}

fn ask_single_choice(max: usize) -> Result<(i32, bool)> {
    loop {
        print!("Ваш ответ (1-{}): ", max);
        let (line, stop) = ask_line()?;
        if stop {
            return Ok((0, true));
        }
        let line = line.trim();
        if is_skip(line) {
            return Ok((-1, false));
        }
        if let Ok(num) = line.parse::<usize>() {
            if num >= 1 && num <= max {
                return Ok((num as i32 - 1, false));
            }
        }
        println!("Введите число от 1 до {}.", max);
    }
}

fn print_header(pool: usize, n: usize) {
    println!("{}", "=".repeat(64));
    println!("ТЕСТ ЭКЗАМЕНА");
    println!("{}", "=".repeat(64));
    println!(
        "Вопросов в базе: {}. В тесте: {} (выбраны случайно).",
        pool, n
    );
    println!("Для вопросов с вариантами введите номер варианта.");
    println!("Для вопросов с множественным выбором введите номера через запятую.");
    println!("Команды: !пропуск — пропустить вопрос, !выход — завершить тест досрочно, !старт - для начала теста.");
}

fn progress_bar(percent: f64) -> String {
    const WIDTH: usize = 40;
    let filled = (percent / 100.0 * WIDTH as f64).round() as usize;
    let filled = filled.min(WIDTH);
    let empty = WIDTH - filled;
    format!("[{}{}]", "#".repeat(filled), ".".repeat(empty))
}

fn print_report(cfg: &Config, results: &[AnswerResult], planned: usize, aborted: bool) {
    println!();
    println!("{}", "=".repeat(64));
    println!("  РЕЗУЛЬТАТЫ");
    println!("{}", "=".repeat(64));

    if results.is_empty() {
        println!("Ни на один вопрос ответа не получено.");
        return;
    }

    let right = results.iter().filter(|r| r.correct).count();
    let wrong: Vec<_> = results.iter().filter(|r| !r.correct).collect();

    let total = planned;
    if aborted {
        println!("Тест прерван: отвечено {} из {} вопросов.", results.len(), planned);
    }
    let percent = right as f64 / total as f64 * 100.0;

    println!("Правильных ответов: {} из {}", right, total);
    println!("Результат: {:.1}% — {}", percent, cfg.grade(percent));
    println!("{}", progress_bar(percent));

    if wrong.is_empty() {
        println!("\nОшибок нет. Отличная работа!");
        return;
    }

    println!("\nНеверно отвеченные вопросы ({}):", wrong.len());
    for (i, r) in wrong.iter().enumerate() {
        println!("{}", "-".repeat(64));
        println!("{}. {}", i + 1, r.question.text);
        println!("   Ваш ответ:        {}", r.user_answer);
        println!("   Правильный ответ: {}", r.question.correct_answer_string());
    }
    println!("{}", "-".repeat(64));
}

fn run_interactive(cfg: &Config, test: Test, delay: Duration) -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut first_run = true;

    loop {
        let n = test.count.min(test.questions.len());

        if first_run {
            print_header(test.questions.len(), n);
            if !wait_for_start()? {
                return Ok(());
            }
            first_run = false;
            clear_screen()?;
        }

        let mut selected: Vec<Question> = test.questions.clone();
        selected.shuffle(&mut rng);
        selected.truncate(n);

        let mut results = Vec::with_capacity(n);
        let mut aborted = false;

        for (i, q) in selected.iter().enumerate() {
            if i > 0 {
                clear_screen()?;
            }

            println!("\nВопрос {} из {}", i + 1, n);
            println!("{}", "-".repeat(64));

            if q.is_choice() {
                if q.is_multiple_choice() {
                    println!("[ВЫБЕРИТЕ НЕСКОЛЬКО ОТВЕТОВ]");
                } else {
                    println!("[ВЫБЕРИТЕ ОДИН ОТВЕТ]");
                }
            } else {
                println!("[ВВЕДИТЕ ТЕКСТОВЫЙ ОТВЕТ]");
            }
            println!("{}", q.text);

            let (user_answer, correct) = if q.is_choice() {
                let mut shuffled = q.options.clone();
                shuffled.shuffle(&mut rng);

                let correct_indices: Vec<usize> = q.correct_indices
                    .iter()
                    .map(|&original_idx| {
                        shuffled.iter().position(|opt| *opt == q.options[original_idx]).unwrap()
                    })
                    .collect();

                println!();
                for (j, opt) in shuffled.iter().enumerate() {
                    println!("  {}) {}", j + 1, opt);
                }

                println!();

                let is_multiple = q.is_multiple_choice();

                let (chosen_indices, stop) = if is_multiple {
                    ask_multiple_choice(shuffled.len())?
                } else {
                    let (choice, stop) = ask_single_choice(shuffled.len())?;
                    if choice >= 0 {
                        (vec![choice as usize], stop)
                    } else {
                        (Vec::new(), stop)
                    }
                };

                if stop {
                    aborted = true;
                    break;
                }

                if !chosen_indices.is_empty() {
                    let answers: Vec<String> = chosen_indices
                        .iter()
                        .map(|&idx| shuffled[idx].clone())
                        .collect();
                    let user_answer_str = answers.join(", ");

                    let ok = if is_multiple {
                        let mut sorted_chosen = chosen_indices.clone();
                        let mut sorted_correct = correct_indices.clone();
                        sorted_chosen.sort();
                        sorted_correct.sort();
                        sorted_chosen == sorted_correct
                    } else {
                        if let Some(&idx) = chosen_indices.first() {
                            idx == correct_indices[0]
                        } else {
                            false
                        }
                    };

                    (user_answer_str, ok)
                } else {
                    ("(пропущено)".to_string(), false)
                }
            } else {
                let (line, stop) = ask_line()?;
                if stop {
                    aborted = true;
                    break;
                }
                let ans = line.trim().to_string();
                let ok = if ans.is_empty() {
                    false
                } else {
                    match_text(&ans, q)
                };
                (ans, ok)
            };

            if aborted {
                break;
            }

            if user_answer.is_empty() {
                let display = if user_answer == "(пропущено)" {
                    "(пропущено)"
                } else {
                    "(нет ответа)"
                };
                results.push(AnswerResult {
                    question: q.clone(),
                    user_answer: display.to_string(),
                    correct: false,
                });
                println!("  Неверно");
            } else {
                let display = if user_answer.trim().is_empty() {
                    "(нет ответа)"
                } else {
                    &user_answer
                };
                results.push(AnswerResult {
                    question: q.clone(),
                    user_answer: display.to_string(),
                    correct,
                });
                if correct {
                    println!("  Верно");
                } else {
                    println!("  Неверно");
                }
            }

            if i < n - 1 {
                std::thread::sleep(delay);
            }
        }

        clear_screen()?;
        print_report(cfg, &results, n, aborted);

        loop {
            print!("\nВведите !рестарт для повторного теста или !выход для выхода: ");
            let (line, stop) = ask_line()?;
            if stop {
                return Ok(());
            }
            let line = line.trim();
            if is_quit(line) {
                return Ok(());
            }
            if is_restart(line) {
                clear_screen()?;
                break;
            }
            println!("Неизвестная команда.");
        }
    }
}
pub fn run(cfg: &Config) -> Result<()> {
    let delay = if cfg.result_delay.is_empty() {
        bail!("поле result_delay не может быть пустым");
    } else {
        humantime::parse_duration(&cfg.result_delay)
            .with_context(|| format!("некорректное значение result_delay={:?}", cfg.result_delay))?
    };

    let pool = parse_questions(&cfg.test_path)
        .with_context(|| format!("ошибка чтения вопросов из файла {:?}", cfg.test_path))?;

    let test = Test {
        questions: pool,
        count: cfg.test_count,
    };

    run_interactive(cfg, test, delay)
}
