use rand::prelude::SliceRandom;
use crate::dto::question::Question;
use crate::types::answer_type::AnswerType;
use crate::utils::answer_handler::{ask_multiple_choice, ask_single_choice, ask_text_answer};
use crate::utils::text_matcher::match_text;

pub fn handle_answer(q: &Question, rng: &mut rand::rngs::ThreadRng) -> anyhow::Result<(String, bool)> {
    let answer_type = q.get_answer_type();
    println!("{}", q.text);

    match answer_type {
        AnswerType::Single => {
            let mut shuffled = q.options.clone();
            shuffled.shuffle(rng);

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
        AnswerType::Multiple => {
            let mut shuffled = q.options.clone();
            shuffled.shuffle(rng);

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
        AnswerType::Text => {
            let (answer, stop) = ask_text_answer()?;
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
