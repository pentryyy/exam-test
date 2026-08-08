use rand::prelude::SliceRandom;
use crate::dto::question::Question;
use crate::types::input_type::InputType;
use crate::utils::input_handler::{ask_multiple_choice, ask_single_choice, ask_text_input};
use crate::utils::text_matcher::match_text;

fn prepare_shuffled_options(q: &Question, rng: &mut rand::rngs::ThreadRng) -> (Vec<String>, Vec<usize>) {
    let mut shuffled = q.options.clone();
    shuffled.shuffle(rng);

    let correct_indices: Vec<usize> = q.correct_indices
        .iter()
        .map(|&original_idx| {
            shuffled.iter().position(|opt| *opt == q.options[original_idx]).unwrap()
        })
        .collect();

    (shuffled, correct_indices)
}

fn display_options(shuffled: &[String]) {
    println!();
    for (j, opt) in shuffled.iter().enumerate() {
        println!("  {}) {}", j + 1, opt);
    }
    println!();
}

fn process_single_choice(shuffled: Vec<String>, correct_indices: Vec<usize>) -> anyhow::Result<(String, bool)> {
    let (choice, stop) = ask_single_choice(shuffled.len())?;
    if stop {
        return Ok((String::new(), false));
    }
    if choice >= 0 {
        let idx = choice as usize;
        let user_answer_str = shuffled[idx].clone();
        let ok = idx == correct_indices[0];
        Ok((user_answer_str, ok))
    } else {
        Ok(("(пропущено)".to_string(), false))
    }
}

fn process_multiple_choice(shuffled: Vec<String>, correct_indices: Vec<usize>) -> anyhow::Result<(String, bool)> {
    let (chosen_indices, stop) = ask_multiple_choice(shuffled.len())?;
    if stop {
        return Ok((String::new(), false));
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

        Ok((user_answer_str, ok))
    } else {
        Ok(("(пропущено)".to_string(), false))
    }
}

pub fn handle_answer(q: &Question, rng: &mut rand::rngs::ThreadRng) -> anyhow::Result<(String, bool)> {
    let answer_type = q.get_answer_type();
    println!("{}", q.text);

    match answer_type {
        InputType::SingleAnswer => {
            let (shuffled, correct_indices) = prepare_shuffled_options(q, rng);
            display_options(&shuffled);
            process_single_choice(shuffled, correct_indices)
        }
        InputType::MultipleAnswer => {
            let (shuffled, correct_indices) = prepare_shuffled_options(q, rng);
            display_options(&shuffled);
            process_multiple_choice(shuffled, correct_indices)
        }
        InputType::TextAnswer => {
            let (answer, stop) = ask_text_input()?;
            if stop {
                return Ok((String::new(), false));
            }
            let ok = if answer.is_empty() {
                false
            } else {
                match_text(&answer, q)
            };
            Ok((answer, ok))
        }
    }
}