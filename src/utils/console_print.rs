use crate::config::config::Grader;
use crate::dto::answer_result::AnswerResult;
use std::io::{self, Write};

pub fn progress_bar(percent: f64) -> String {
    const WIDTH: usize = 40;
    let filled = (percent / 100.0 * WIDTH as f64).round() as usize;
    let filled = filled.min(WIDTH);
    let empty = WIDTH - filled;
    format!("[{}{}]", "#".repeat(filled), ".".repeat(empty))
}

pub fn print_header_to<W: Write>(
    output: &mut W,
    subject: &str,
    pool: usize,
    n: usize,
) -> io::Result<()> {
    writeln!(output, "{}", "=".repeat(64))?;
    writeln!(output, "ТЕСТ ЭКЗАМЕНА {}", subject)?;
    writeln!(output, "{}", "=".repeat(64))?;
    writeln!(
        output,
        "Вопросов в базе: {}. В тесте: {} (выбраны случайно).",
        pool, n
    )?;
    writeln!(
        output,
        "Команды: !пропуск — пропустить вопрос, !выход — завершить тест досрочно, !старт - для начала теста."
    )?;
    Ok(())
}

pub fn print_header(subject: &str, pool: usize, n: usize) {
    let mut stdout = io::stdout();
    let _ = print_header_to(&mut stdout, subject, pool, n);
}

pub fn print_report_to<W: Write, G: Grader>(
    output: &mut W,
    results: &[AnswerResult],
    planned: usize,
    aborted: bool,
    grader: &G,
) -> io::Result<()> {
    writeln!(output)?;
    writeln!(output, "{}", "=".repeat(64))?;
    writeln!(output, "  РЕЗУЛЬТАТЫ")?;
    writeln!(output, "{}", "=".repeat(64))?;

    if results.is_empty() {
        writeln!(output, "Ни на один вопрос ответа не получено.")?;
        return Ok(());
    }

    let right = results.iter().filter(|r| r.correct).count();
    let wrong: Vec<_> = results.iter().filter(|r| !r.correct).collect();

    let total = planned;
    if aborted {
        writeln!(
            output,
            "Тест прерван: отвечено {} из {} вопросов.",
            results.len(),
            planned
        )?;
    }
    let percent = right as f64 / total as f64 * 100.0;

    writeln!(output, "Правильных ответов: {} из {}", right, total)?;
    writeln!(
        output,
        "Результат: {:.1}% — {}",
        percent,
        grader.grade(percent)
    )?;
    writeln!(output, "{}", progress_bar(percent))?;

    if wrong.is_empty() {
        writeln!(output, "\nОшибок нет. Отличная работа!")?;
        return Ok(());
    }

    writeln!(output, "\nНеверно отвеченные вопросы ({}):", wrong.len())?;
    for (i, r) in wrong.iter().enumerate() {
        writeln!(output, "{}", "-".repeat(64))?;
        writeln!(output, "{}. {}", i + 1, r.question.question)?;
        writeln!(output, "   Ваш ответ:        {}", r.user_answer)?;
        writeln!(
            output,
            "   Правильный ответ: {}",
            r.question.correct_answer_string()
        )?;
    }
    writeln!(output, "{}", "-".repeat(64))?;
    Ok(())
}

pub fn print_report<G: Grader>(
    results: &[AnswerResult],
    planned: usize,
    aborted: bool,
    grader: &G,
) {
    let mut stdout = io::stdout();
    let _ = print_report_to(&mut stdout, results, planned, aborted, grader);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::test::CfgTest;
    use crate::types::answer_type::AnswerType;

    #[test]
    fn progress_bar_0_percent() {
        assert_eq!(
            progress_bar(0.0),
            "[........................................]"
        );
    }

    #[test]
    fn progress_bar_50_percent() {
        assert_eq!(
            progress_bar(50.0),
            "[####################....................]"
        );
    }

    #[test]
    fn progress_bar_100_percent() {
        assert_eq!(
            progress_bar(100.0),
            "[########################################]"
        );
    }

    #[test]
    fn progress_bar_rounding() {
        assert_eq!(
            progress_bar(42.5),
            "[#################.......................]"
        );
        assert_eq!(
            progress_bar(42.4),
            "[#################.......................]"
        );
        assert_eq!(
            progress_bar(42.49),
            "[#################.......................]"
        );
        assert_eq!(
            progress_bar(42.5),
            "[#################.......................]"
        );
        assert_eq!(
            progress_bar(42.51),
            "[#################.......................]"
        );
        assert_eq!(
            progress_bar(42.99),
            "[#################.......................]"
        );
        assert_eq!(
            progress_bar(43.0),
            "[#################.......................]"
        );
        assert_eq!(
            progress_bar(43.1),
            "[#################.......................]"
        );
        assert_eq!(
            progress_bar(43.75),
            "[##################......................]"
        );
    }

    #[test]
    fn print_header_to_creates_correct_output() {
        let mut output = Vec::new();
        let subject = "Информатика";
        let pool = 150;
        let n = 10;
        print_header_to(&mut output, subject, pool, n).unwrap();
        let out = String::from_utf8(output).unwrap();
        let expected = format!(
            "{}\nТЕСТ ЭКЗАМЕНА {}\n{}\nВопросов в базе: {}. В тесте: {} (выбраны случайно).\nКоманды: !пропуск — пропустить вопрос, !выход — завершить тест досрочно, !старт - для начала теста.\n",
            "=".repeat(64),
            subject,
            "=".repeat(64),
            pool,
            n
        );
        assert_eq!(out, expected);
    }

    struct MockGrader;
    impl Grader for MockGrader {
        fn grade(&self, percent: f64) -> String {
            if percent >= 80.0 {
                "Отлично".to_string()
            } else if percent >= 60.0 {
                "Хорошо".to_string()
            } else {
                "Плохо".to_string()
            }
        }
    }

    fn make_result(question: &str, user_answer: &str, correct: bool) -> AnswerResult {
        let cfg_test = CfgTest {
            question: question.to_string(),
            options: vec![],
            correct: AnswerType::Text("Правильный".to_string()),
            accept: vec![],
        };
        AnswerResult {
            question: cfg_test,
            user_answer: user_answer.to_string(),
            correct,
        }
    }

    #[test]
    fn print_report_to_with_empty_results() {
        let mut output = Vec::new();
        let results = vec![];
        let grader = MockGrader;
        print_report_to(&mut output, &results, 5, false, &grader).unwrap();
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Ни на один вопрос ответа не получено."));
    }

    #[test]
    fn print_report_to_with_all_correct() {
        let mut output = Vec::new();
        let results = vec![make_result("Q1", "A", true), make_result("Q2", "B", true)];
        let grader = MockGrader;
        print_report_to(&mut output, &results, 2, false, &grader).unwrap();
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Правильных ответов: 2 из 2"));
        assert!(out.contains("Результат: 100.0% — Отлично"));
        assert!(out.contains("Ошибок нет. Отличная работа!"));
    }

    #[test]
    fn print_report_to_with_wrong_answers() {
        let mut output = Vec::new();
        let results = vec![
            make_result("Вопрос 1", "Неправильно", false),
            make_result("Вопрос 2", "Правильно", true),
        ];
        let grader = MockGrader;
        print_report_to(&mut output, &results, 2, false, &grader).unwrap();
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Правильных ответов: 1 из 2"));
        assert!(out.contains("Результат: 50.0% — Плохо"));
        assert!(out.contains("Неверно отвеченные вопросы (1):"));
        assert!(out.contains("1. Вопрос 1"));
        assert!(out.contains("Ваш ответ:        Неправильно"));
        assert!(out.contains("Правильный ответ: Правильный"));
    }

    #[test]
    fn print_report_to_with_aborted() {
        let mut output = Vec::new();
        let results = vec![make_result("Q1", "A", true)];
        let grader = MockGrader;
        print_report_to(&mut output, &results, 5, true, &grader).unwrap();
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Тест прерван: отвечено 1 из 5 вопросов."));
        assert!(out.contains("Правильных ответов: 1 из 5"));
        assert!(out.contains("Результат: 20.0% — Плохо"));
    }
}
