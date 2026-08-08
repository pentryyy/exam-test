use crate::config::config::Config;
use crate::dto::answer_result::AnswerResult;
use crate::dto::question::Question;
use crate::dto::test::Test;
use crate::utils::console_clear::{clear_screen, flush_screen};
use crate::utils::input_action::handle_answer;
use crate::utils::input_handler::ask_line;
use crate::utils::question_parser::parse_questions;
use anyhow::{bail, Context, Result};
use rand::prelude::SliceRandom;
use std::time::Duration;
use crate::types::console_operation_type::ConsoleOperation;
use crate::utils::console_print::{print_header, print_report};

pub struct CliApp<'a> {
    cfg: &'a Config,
}

impl<'a> CliApp<'a> {
    pub fn new(cfg: &'a Config) -> Self {
        CliApp { cfg }
    }

    pub fn run(&self) -> Result<()> {
        let delay = if self.cfg.result_delay.is_empty() {
            bail!("поле result_delay не может быть пустым");
        } else {
            humantime::parse_duration(&self.cfg.result_delay)
                .with_context(|| format!("некорректное значение result_delay={:?}", self.cfg.result_delay))?
        };

        let pool = parse_questions(&self.cfg.test_path)
            .with_context(|| format!("ошибка чтения вопросов из файла {:?}", self.cfg.test_path))?;

        let test = Test {
            questions: pool,
            count: self.cfg.test_count,
        };

        self.run_interactive(test, delay)
    }

    fn wait_for_start(&self) -> Result<bool> {
        loop {
            flush_screen()?;

            let (line, stop) = ask_line()?;
            if stop {
                return Ok(false);
            }

            match ConsoleOperation::from_input(&line) {
                ConsoleOperation::Start => return Ok(true),
                ConsoleOperation::Quit => return Ok(false),
                _ => println!("Неизвестная команда. Введите !старт или !выход."),
            }
        }
    }

    fn handle_question(
        &self,
        q: &Question,
        index: usize,
        total: usize,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Result<(AnswerResult, bool)> {
        println!("\nВопрос {} из {}", index + 1, total);
        println!("{}", "-".repeat(64));

        let (user_answer, correct) = handle_answer(q, rng)?;

        if user_answer.is_empty() && !correct {
            return Ok((
                AnswerResult {
                    question: q.clone(),
                    user_answer: "(пропущено)".to_string(),
                    correct: false,
                },
                true, // true = пропущено/прервано
            ));
        }

        let display = if user_answer.trim().is_empty() {
            "(нет ответа)"
        } else {
            &user_answer
        };

        let result = AnswerResult {
            question: q.clone(),
            user_answer: display.to_string(),
            correct,
        };

        if correct {
            println!("  Верно");
        } else {
            println!("  Неверно");
        }

        Ok((result, false))
    }

    fn handle_restart_prompt(&self) -> Result<bool> {
        loop {
            print!("\nВведите !рестарт для повторного теста или !выход для выхода: ");
            flush_screen()?;

            let (line, stop) = ask_line()?;
            if stop {
                return Ok(false);
            }

            match ConsoleOperation::from_input(&line) {
                ConsoleOperation::Restart => return Ok(true),
                ConsoleOperation::Quit => return Ok(false),
                _ => print!("Неизвестная команда. Введите !рестарт или !выход."),
            }
        }
    }

    fn run_interactive(&self, test: Test, delay: Duration) -> Result<()> {
        let mut rng = rand::thread_rng();
        let mut first_run = true;

        loop {
            let n = test.count.min(test.questions.len());

            if first_run {
                print_header(test.questions.len(), n);
                if !self.wait_for_start()? {
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

                let (result, skipped) = self.handle_question(q, i, n, &mut rng)?;
                results.push(result);

                if skipped {
                    aborted = true;
                    break;
                }

                if i < n - 1 {
                    std::thread::sleep(delay);
                }
            }

            clear_screen()?;
            print_report(&results, n, aborted, self.cfg);

            if !self.handle_restart_prompt()? {
                return Ok(());
            }
            clear_screen()?;
        }
    }
}
