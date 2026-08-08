use crate::config::config::Config;
use crate::dto::answer_result::AnswerResult;
use crate::dto::question::Question;
use crate::dto::test::Test;
use crate::utils::input_handler::{ask_line, is_quit, is_restart, wait_for_start};
use crate::utils::input_action::handle_answer;
use crate::utils::clr_operation::{clear_screen, flush_screen};
use crate::utils::question_parser::parse_questions;
use anyhow::{bail, Context, Result};
use rand::prelude::SliceRandom;
use std::time::Duration;

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

            let (user_answer, correct) = handle_answer(q, &mut rng)?;
            if user_answer.is_empty() && correct == false {
                aborted = true;
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
            flush_screen()?;

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
