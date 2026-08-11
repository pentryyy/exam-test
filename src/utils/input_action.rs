use crate::dto::test::CfgTest;
use crate::types::answer_type::{AnswerKind, AnswerType};
use crate::utils::input_handler::{ask_multiple_choice_from, ask_single_choice_from, ask_text_input_from};
use crate::utils::text_matcher::match_text;
use anyhow::Result;
use rand::prelude::SliceRandom;
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use rand::Rng;

pub fn handle_answer(test: &CfgTest, rng: &mut rand::rngs::ThreadRng) -> Result<(String, bool, bool)> {
    let answer_type = test.get_answer_kind();
    println!("{}", test.question);

    match answer_type {
        AnswerKind::SingleAnswer => {
            let (shuffled, correct_indices) = prepare_shuffled_options(test, rng);
            display_options(&shuffled);
            let mut input = BufReader::new(stdin());
            let mut output = stdout();
            process_single_choice_from(&mut input, &mut output, shuffled, correct_indices)
        }
        AnswerKind::MultipleAnswer => {
            let (shuffled, correct_indices) = prepare_shuffled_options(test, rng);
            display_options(&shuffled);
            let mut input = BufReader::new(stdin());
            let mut output = stdout();
            process_multiple_choice_from(&mut input, &mut output, shuffled, correct_indices)
        }
        AnswerKind::TextAnswer => {
            let mut input = BufReader::new(stdin());
            let mut output = stdout();
            process_text_answer_from(&mut input, &mut output, test)
        }
    }
}

pub fn process_single_choice_from<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    shuffled: Vec<String>,
    correct_indices: Vec<usize>,
) -> Result<(String, bool, bool)> {
    let (choice, stop) = ask_single_choice_from(input, output, shuffled.len())?;
    if stop {
        return Ok((String::new(), false, true));
    }
    if choice >= 0 {
        let idx = choice as usize;
        let user_answer_str = shuffled[idx].clone();
        let ok = idx == correct_indices[0];
        Ok((user_answer_str, ok, false))
    } else {
        Ok((String::new(), false, true))
    }
}

pub fn process_multiple_choice_from<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    shuffled: Vec<String>,
    correct_indices: Vec<usize>,
) -> Result<(String, bool, bool)> {
    let (chosen_indices, stop) = ask_multiple_choice_from(input, output, shuffled.len())?;
    if stop {
        return Ok((String::new(), false, true));
    }

    if !chosen_indices.is_empty() {
        let user_answer_str = chosen_indices
            .iter()
            .map(|&idx| shuffled[idx].clone())
            .collect::<Vec<String>>()
            .join(", ");

        let mut sorted_chosen = chosen_indices.clone();
        let mut sorted_correct = correct_indices.clone();
        sorted_chosen.sort();
        sorted_correct.sort();
        let ok = sorted_chosen == sorted_correct;

        Ok((user_answer_str, ok, false))
    } else {
        Ok((String::new(), false, true))
    }
}

pub fn process_text_answer_from<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    test: &CfgTest,
) -> Result<(String, bool, bool)> {
    let (answer, stop) = ask_text_input_from(input, output)?;
    if stop {
        return Ok((String::new(), false, true));
    }
    if answer.is_empty() {
        return Ok((String::new(), false, true));
    }
    let ok = match_text(&answer, test);
    Ok((answer, ok, false))
}

fn prepare_shuffled_options<R: Rng>(q: &CfgTest, rng: &mut R) -> (Vec<String>, Vec<usize>) {
    let original_correct_indices: Vec<usize> = match &q.correct {
        AnswerType::SingleAnswer(idx) => vec![*idx],
        AnswerType::MultipleAnswer(indices) => indices.iter().copied().collect(),
        AnswerType::TextAnswer(_) => return (q.options.iter().map(|opt| opt.answer.clone()).collect(), Vec::new()),
    };

    let mut indexed: Vec<(usize, String)> = q.options.iter().map(|opt| opt.answer.clone()).enumerate().collect();
    indexed.shuffle(rng);

    let mut shuffled_options = Vec::with_capacity(indexed.len());
    let mut original_to_shuffled = vec![0; q.options.len()];

    for (new_pos, (orig_idx, value)) in indexed.into_iter().enumerate() {
        shuffled_options.push(value);
        original_to_shuffled[orig_idx] = new_pos;
    }

    let shuffled_correct_indices: Vec<usize> = original_correct_indices
        .iter()
        .map(|&orig_idx| original_to_shuffled[orig_idx])
        .collect();

    (shuffled_options, shuffled_correct_indices)
}

fn display_options(shuffled: &[String]) {
    println!();
    for (j, opt) in shuffled.iter().enumerate() {
        println!("  {}) {}", j + 1, opt);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::test::CfgOption;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::io::Cursor;

    fn make_test(options: Vec<&str>, correct: AnswerType) -> CfgTest {
        CfgTest {
            question: "?".to_string(),
            options: options.into_iter().map(|s| CfgOption { answer: s.to_string() }).collect(),
            correct,
            accept: vec![],
        }
    }

    #[test]
    fn prepare_shuffled_options_preserves_elements() {
        let test = make_test(
            vec!["A", "B", "C", "D"],
            AnswerType::SingleAnswer(0),
        );
        let mut rng = StdRng::seed_from_u64(42);
        let (shuffled, correct) = prepare_shuffled_options(&test, &mut rng);
        let mut sorted_orig = test.options.iter().map(|o| o.answer.clone()).collect::<Vec<_>>();
        sorted_orig.sort();
        let mut sorted_shuf = shuffled.clone();
        sorted_shuf.sort();
        assert_eq!(sorted_orig, sorted_shuf);
        assert_eq!(shuffled[correct[0]], "A");
    }

    #[test]
    fn prepare_shuffled_options_multiple_correct() {
        let test = make_test(
            vec!["X", "Y", "Z", "W"],
            AnswerType::MultipleAnswer(vec![0, 2].into_iter().collect()),
        );
        let mut rng = StdRng::seed_from_u64(123);
        let (shuffled, correct) = prepare_shuffled_options(&test, &mut rng);
        let mut correct_texts: Vec<_> = correct.iter().map(|&i| shuffled[i].clone()).collect();
        correct_texts.sort();
        assert_eq!(correct_texts, vec!["X", "Z"]);
    }

    #[test]
    fn process_single_choice_from_valid() {
        let shuffled = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let correct = vec![1];
        let mut input = Cursor::new("2\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_single_choice_from(&mut input, &mut output, shuffled, correct).unwrap();
        assert_eq!(answer, "B");
        assert!(ok);
        assert!(!skipped);
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Ваш ответ (1-3):"));
    }

    #[test]
    fn process_single_choice_from_skip() {
        let shuffled = vec!["A".to_string(), "B".to_string()];
        let correct = vec![0];
        let mut input = Cursor::new("!пропуск\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_single_choice_from(&mut input, &mut output, shuffled, correct).unwrap();
        assert_eq!(answer, "");
        assert!(!ok);
        assert!(skipped);
    }

    #[test]
    fn process_single_choice_from_quit() {
        let shuffled = vec!["A".to_string(), "B".to_string()];
        let correct = vec![0];
        let mut input = Cursor::new("!выход\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_single_choice_from(&mut input, &mut output, shuffled, correct).unwrap();
        assert_eq!(answer, "");
        assert!(!ok);
        assert!(skipped);
    }

    #[test]
    fn process_single_choice_from_invalid_then_valid() {
        let shuffled = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let correct = vec![2]; // "C"
        let mut input = Cursor::new("0\n3\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_single_choice_from(&mut input, &mut output, shuffled, correct).unwrap();
        assert_eq!(answer, "C");
        assert!(ok);
        assert!(!skipped);
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Введите число от 1 до 3"));
    }

    #[test]
    fn process_multiple_choice_from_valid() {
        let shuffled = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let correct = vec![0, 2];
        let mut input = Cursor::new("1, 3\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_multiple_choice_from(&mut input, &mut output, shuffled, correct).unwrap();
        assert_eq!(answer, "A, C");
        assert!(ok);
        assert!(!skipped);
    }

    #[test]
    fn process_multiple_choice_from_wrong_order() {
        let shuffled = vec!["X".to_string(), "Y".to_string(), "Z".to_string()];
        let correct = vec![1, 2];
        let mut input = Cursor::new("3, 2\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_multiple_choice_from(&mut input, &mut output, shuffled, correct).unwrap();
        assert_eq!(answer, "Z, Y");
        assert!(ok);
        assert!(!skipped);
    }

    #[test]
    fn process_multiple_choice_from_skip() {
        let shuffled = vec!["A".to_string(), "B".to_string()];
        let correct = vec![0, 1];
        let mut input = Cursor::new("!пропуск\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_multiple_choice_from(&mut input, &mut output, shuffled, correct).unwrap();
        assert_eq!(answer, "");
        assert!(!ok);
        assert!(skipped);
    }

    #[test]
    fn process_multiple_choice_from_quit() {
        let shuffled = vec!["A".to_string(), "B".to_string()];
        let correct = vec![0];
        let mut input = Cursor::new("!выход\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_multiple_choice_from(&mut input, &mut output, shuffled, correct).unwrap();
        assert_eq!(answer, "");
        assert!(!ok);
        assert!(skipped);
    }

    #[test]
    fn process_text_answer_from_valid() {
        let test = CfgTest {
            question: "?".to_string(),
            options: vec![],
            correct: AnswerType::TextAnswer("правильный".to_string()),
            accept: vec![],
        };
        let mut input = Cursor::new("правильный\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_text_answer_from(&mut input, &mut output, &test).unwrap();
        assert_eq!(answer, "правильный");
        assert!(ok);
        assert!(!skipped);
    }

    #[test]
    fn process_text_answer_from_empty_then_valid() {
        let test = CfgTest {
            question: "?".to_string(),
            options: vec![],
            correct: AnswerType::TextAnswer("текст".to_string()),
            accept: vec![],
        };
        let mut input = Cursor::new("\nтекст\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_text_answer_from(&mut input, &mut output, &test).unwrap();
        assert_eq!(answer, "текст");
        assert!(ok);
        assert!(!skipped);
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Ответ не может быть пустым"));
    }

    #[test]
    fn process_text_answer_from_skip() {
        let test = CfgTest {
            question: "?".to_string(),
            options: vec![],
            correct: AnswerType::TextAnswer("текст".to_string()),
            accept: vec![],
        };
        let mut input = Cursor::new("!пропуск\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_text_answer_from(&mut input, &mut output, &test).unwrap();
        assert_eq!(answer, "");
        assert!(!ok);
        assert!(skipped);
    }

    #[test]
    fn process_text_answer_from_quit() {
        let test = CfgTest {
            question: "?".to_string(),
            options: vec![],
            correct: AnswerType::TextAnswer("текст".to_string()),
            accept: vec![],
        };
        let mut input = Cursor::new("!выход\n".as_bytes());
        let mut output = Vec::new();
        let (answer, ok, skipped) = process_text_answer_from(&mut input, &mut output, &test).unwrap();
        assert_eq!(answer, "");
        assert!(!ok);
        assert!(skipped);
    }
}
