use crate::dto::test::CfgTest;
use crate::types::answer_type::{AnswerKind, AnswerType};
use crate::utils::input_handler::{ask_multiple_choice, ask_single_choice, ask_text_input};
use crate::utils::text_matcher::match_text;
use anyhow::Result;
use rand::prelude::SliceRandom;

fn prepare_shuffled_options(q: &CfgTest, rng: &mut rand::rngs::ThreadRng) -> (Vec<String>, Vec<usize>) {
    let original_correct_indices: Vec<usize> = match &q.correct {
        AnswerType::SingleAnswer(idx) => vec![*idx],
        AnswerType::MultipleAnswer(indices) => indices.clone(),
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

fn process_single_choice(shuffled: Vec<String>, correct_indices: Vec<usize>) -> Result<(String, bool, bool)> {
    let (choice, stop) = ask_single_choice(shuffled.len())?;
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

fn process_multiple_choice(shuffled: Vec<String>, correct_indices: Vec<usize>) -> Result<(String, bool, bool)> {
    let (chosen_indices, stop) = ask_multiple_choice(shuffled.len())?;
    if stop {
        return Ok((String::new(), false, true));
    }

    if !chosen_indices.is_empty() {
        let answers: Vec<String> = chosen_indices
            .iter()
            .map(|&idx| shuffled[idx].clone())
            .collect();
        let user_answer_str = answers.join(", ");

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

fn process_text_answer(test: &CfgTest) -> Result<(String, bool, bool)> {
    let (answer, stop) = ask_text_input()?;
    if stop {
        return Ok((String::new(), false, true));
    }
    let ok = if answer.is_empty() {
        false
    } else {
        match_text(&answer, test)
    };
    Ok((answer, ok, false))
}

pub fn handle_answer(test: &CfgTest, rng: &mut rand::rngs::ThreadRng) -> Result<(String, bool, bool)> {
    let answer_type = test.get_answer_type();
    println!("{}", test.question);

    match answer_type {
        AnswerKind::SingleAnswer => {
            let (shuffled, correct_indices) = prepare_shuffled_options(test, rng);
            display_options(&shuffled);
            process_single_choice(shuffled, correct_indices)
        }
        AnswerKind::MultipleAnswer => {
            let (shuffled, correct_indices) = prepare_shuffled_options(test, rng);
            display_options(&shuffled);
            process_multiple_choice(shuffled, correct_indices)
        }
        AnswerKind::TextAnswer => process_text_answer(test),
    }
}
