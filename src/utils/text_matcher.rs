use crate::dto::question::Question;
use crate::types::answer_type::AnswerType;

pub fn match_text(user: &str, q: &Question) -> bool {
    let nu = normalize(user);
    if nu.is_empty() {
        return false;
    }

    let correct_text = match &q.correct {
        AnswerType::TextAnswer(text) => text.clone(),
        _ => return false,
    };

    let mut variants = Vec::with_capacity(1 + q.accept.len());
    variants.push(correct_text);
    variants.extend(q.accept.clone());

    for v in variants {
        let nv = normalize(&v);
        if nv.is_empty() {
            continue;
        }
        if nu == nv {
            return true;
        }
        if nv.chars().count() >= 8 && levenshtein(&nu, &nv) <= 1 {
            return true;
        }
    }
    false
}

fn normalize(s: &str) -> String {
    let lower = s.trim().to_lowercase();
    let mut result = String::with_capacity(lower.len());

    for c in lower.chars() {
        let ch = if c == 'ё' { 'е' } else { c };
        if ch.is_alphabetic() || ch.is_digit(10) {
            result.push(ch);
        } else {
            result.push(' ');
        }
    }

    result
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let len_a = a_chars.len();
    let len_b = b_chars.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut prev: Vec<usize> = (0..=len_b).collect();
    let mut cur = vec![0; len_b + 1];

    for i in 1..=len_a {
        cur[0] = i;
        for j in 1..=len_b {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            let del = prev[j] + 1;
            let ins = cur[j - 1] + 1;
            let sub = prev[j - 1] + cost;
            cur[j] = del.min(ins).min(sub);
        }
        std::mem::swap(&mut prev, &mut cur);
    }

    prev[len_b]
}
