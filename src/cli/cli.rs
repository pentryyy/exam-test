use crate::config::config::Config;
use crate::dto::answer_result::AnswerResult;
use crate::dto::test::{CfgTest, CfgTests, CliTests};
use crate::types::console_operation_type::ConsoleOperation;
use crate::utils::console_clear::{clear_screen, flush_screen};
use crate::utils::console_print::{print_header, print_report};
use crate::utils::input_action::handle_answer;
use crate::utils::input_handler::ask_line;
use anyhow::{Context, Result};
use rand::prelude::SliceRandom;
use std::time::Duration;

type ShouldStopTest = bool;

pub struct CliApp<'a> {
    cfg: &'a Config,
}

impl<'a> CliApp<'a> {
    pub fn new(cfg: &'a Config) -> Self {
        CliApp { cfg }
    }

    pub fn run(&self) -> Result<()> {
        let cfg_tests = CfgTests::load(&self.cfg.test_path)
            .with_context(|| format!("ошибка чтения вопросов из файла {:?}", self.cfg.test_path))?;

        let cli_tests = CliTests {
            questions: cfg_tests.questions,
            count: self.cfg.test_count,
        };

        self.run_interactive(&cfg_tests.exam_subject, cli_tests, self.cfg.result_delay)
    }

    fn wait_for_start(&self) -> Result<ShouldStopTest> {
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
        test: &CfgTest,
        index: usize,
        total: usize,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Result<(AnswerResult, ShouldStopTest)> {
        println!("\nВопрос {} из {}", index + 1, total);
        println!("{}", "-".repeat(64));

        let (user_answer, correct, skipped) = handle_answer(test, rng)?;

        if skipped {
            return Ok((
                AnswerResult {
                    question: test.clone(),
                    user_answer: "(пропущено)".to_string(),
                    correct: false,
                },
                true,
            ));
        }

        let display = if user_answer.trim().is_empty() {
            "(нет ответа)"
        } else {
            &user_answer
        };

        let result = AnswerResult {
            question: test.clone(),
            user_answer: display.to_string(),
            correct,
        };

        result.log_answer();

        Ok((result, false))
    }

    fn handle_restart_prompt(&self) -> Result<ShouldStopTest> {
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

    fn run_interactive(&self, subject: &str, test: CliTests, delay: Duration) -> Result<()> {
        let mut rng = rand::thread_rng();
        let mut first_run = true;

        loop {
            let n = test.count.min(test.questions.len());

            if first_run {
                print_header(subject, test.questions.len(), n);
                if !self.wait_for_start()? {
                    return Ok(());
                }
                first_run = false;
                clear_screen()?;
            }

            let mut indices: Vec<usize> = (0..test.questions.len()).collect();
            indices.shuffle(&mut rng);
            let selected_indices = &indices[0..n];

            let mut results = Vec::with_capacity(n);
            let mut aborted = false;

            for (i, &idx) in selected_indices.iter().enumerate() {
                if i > 0 {
                    clear_screen()?;
                }

                let question = &test.questions[idx];
                let (result, skipped) = self.handle_question(question, i, n, &mut rng)?;
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
