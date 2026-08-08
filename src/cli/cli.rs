use anyhow::{bail, Context, Result};
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;
use clap::Parser;
use rand::prelude::SliceRandom;
use crate::config::config::Config;
use crate::dto::answer_result::AnswerResult;
use crate::dto::question::Question;
use crate::dto::test::Test;
use crate::util::question_parser::parse_questions;
use crate::util::text_matcher::match_text;

fn clear_screen() {
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd").args(["/c", "cls"]).status();
    } else {
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();
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
    low == "!пропуск" || low == "!skip" || s.is_empty()
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

fn ask_choice(max: usize) -> Result<(i32, bool)> {
    loop {
        print!("Ваш ответ (1-{}): ", max);
        io::stdout().flush()?;
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
        println!("Тест прерван: отвечено {} из {} вопросов.", total, planned);
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
        if n == 0 {
            bail!("Количество вопросов должно быть больше нуля");
        }

        if first_run {
            print_header(test.questions.len(), n);
            if !wait_for_start()? {
                return Ok(());
            }
            first_run = false;
            clear_screen();
        }

        let mut selected: Vec<Question> = test.questions.clone();
        selected.shuffle(&mut rng);
        selected.truncate(n);

        let mut results = Vec::with_capacity(n);
        let mut aborted = false;

        for (i, q) in selected.iter().enumerate() {
            if i > 0 {
                clear_screen();
            }

            println!("\nВопрос {} из {}", i + 1, n);
            println!("{}", "-".repeat(64));
            println!("{}", q.text);

            let (user_answer, correct) = if q.is_choice() {
                let mut shuffled = q.options.clone();
                shuffled.shuffle(&mut rng);
                let correct_pos = shuffled
                    .iter()
                    .position(|opt| *opt == q.options[q.correct_index.unwrap()])
                    .unwrap();

                println!();
                for (j, opt) in shuffled.iter().enumerate() {
                    println!("  {}) {}", j + 1, opt);
                }
                let (choice, stop) = ask_choice(shuffled.len())?;
                if stop {
                    aborted = true;
                    break;
                }
                if choice >= 0 {
                    let idx = choice as usize;
                    let ans = shuffled[idx].clone();
                    let ok = idx == correct_pos;
                    (ans, ok)
                } else {
                    ("(пропущено)".to_string(), false)
                }
            } else {
                println!("(введите ответ текстом)");
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

        print_report(cfg, &results, n, aborted);

        loop {
            print!("\nВведите !рестарт для повторного теста или !выход для выхода: ");
            io::stdout().flush()?;
            let (line, stop) = ask_line()?;
            if stop {
                return Ok(());
            }
            let line = line.trim();
            if is_quit(line) {
                return Ok(());
            }
            if is_restart(line) {
                clear_screen();
                break;
            }
            println!("Неизвестная команда.");
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {

    #[arg(short, long)]
    file: Option<String>,

    #[arg(short = 'n', long)]
    count: Option<usize>,
}

pub fn run(cfg: &Config) -> Result<()> {
    use clap::Parser;

    let args = Args::parse();

    let final_file = args.file.unwrap_or_else(|| cfg.test_path.clone());
    let final_count = args.count.unwrap_or(cfg.test_count);

    let delay = if cfg.result_delay.is_empty() {
        Duration::from_millis(500)
    } else {
        humantime::parse_duration(&cfg.result_delay)
            .with_context(|| format!("некорректное значение result_delay={:?}", cfg.result_delay))?
    };

    let pool = parse_questions(&final_file)
        .with_context(|| format!("ошибка чтения вопросов из файла {:?}", final_file))?;

    let test = Test {
        questions: pool,
        count: final_count,
    };

    run_interactive(cfg, test, delay)
}
