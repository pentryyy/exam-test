use crate::config::config::Grader;
use crate::dto::answer_result::AnswerResult;

pub fn progress_bar(percent: f64) -> String {
    const WIDTH: usize = 40;
    let filled = (percent / 100.0 * WIDTH as f64).round() as usize;
    let filled = filled.min(WIDTH);
    let empty = WIDTH - filled;
    format!("[{}{}]", "#".repeat(filled), ".".repeat(empty))
}

pub fn print_header(subject: &str, pool: usize, n: usize) {
    println!("{}", "=".repeat(64));
    println!("ТЕСТ ЭКЗАМЕНА {}", subject);
    println!("{}", "=".repeat(64));
    println!(
        "Вопросов в базе: {}. В тесте: {} (выбраны случайно).",
        pool, n
    );
    println!("Команды: !пропуск — пропустить вопрос, !выход — завершить тест досрочно, !старт - для начала теста.");
}

pub fn print_report<G: Grader>(
    results: &[AnswerResult],
    planned: usize,
    aborted: bool,
    grader: &G
) {
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
    println!("Результат: {:.1}% — {}", percent, grader.grade(percent));
    println!("{}", progress_bar(percent));

    if wrong.is_empty() {
        println!("\nОшибок нет. Отличная работа!");
        return;
    }

    println!("\nНеверно отвеченные вопросы ({}):", wrong.len());
    for (i, r) in wrong.iter().enumerate() {
        println!("{}", "-".repeat(64));
        println!("{}. {}", i + 1, r.question.question);
        println!("   Ваш ответ:        {}", r.user_answer);
        println!("   Правильный ответ: {}", r.question.correct_answer_string());
    }
    println!("{}", "-".repeat(64));
}
