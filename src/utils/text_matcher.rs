use crate::dto::test::CfgTest;
use crate::types::answer_type::AnswerType;

pub fn match_text(user: &str, test: &CfgTest) -> bool {
    let nu = normalize(user);
    if nu.is_empty() {
        return false;
    }

    let correct_text = match &test.correct {
        AnswerType::TextAnswer(text) => text.clone(),
        _ => return false,
    };

    let mut variants = Vec::with_capacity(1 + test.accept.len());
    variants.push(correct_text);
    variants.extend(test.accept.clone());

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    struct TestCfgTest {
        correct: AnswerType,
        accept: Vec<String>,
    }

    impl TestCfgTest {
        fn text_answer(correct: &str, accept: Vec<String>) -> Self {
            Self {
                correct: AnswerType::TextAnswer(correct.to_string()),
                accept,
            }
        }

        fn single_answer() -> Self {
            Self {
                correct: AnswerType::SingleAnswer(0),
                accept: vec![],
            }
        }

        fn multiple_answer() -> Self {
            let mut set = HashSet::new();
            set.insert(0);
            set.insert(1);
            Self {
                correct: AnswerType::MultipleAnswer(set),
                accept: vec![],
            }
        }

        fn to_cfg_test(&self) -> CfgTest {
            CfgTest {
                question: String::new(),
                options: vec![],
                correct: match &self.correct {
                    AnswerType::TextAnswer(s) => AnswerType::TextAnswer(s.clone()),
                    AnswerType::SingleAnswer(idx) => AnswerType::SingleAnswer(*idx),
                    AnswerType::MultipleAnswer(set) => AnswerType::MultipleAnswer(set.clone()),
                },
                accept: self.accept.clone(),
            }
        }
    }

    #[test]
    fn normalize_empty() {
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("   "), "");
        assert_eq!(normalize("\t\n"), "");
    }

    #[test]
    fn normalize_lowercase_and_trim() {
        assert_eq!(normalize("ПриВет"), "привет");
        assert_eq!(normalize("  ТеСт  "), "тест");
        assert_eq!(normalize("Ёжик"), "ежик");
        assert_eq!(normalize("ЁЛКА"), "елка");
    }

    #[test]
    fn normalize_punctuation_and_spaces() {
        assert_eq!(normalize("привет, мир!"), "привет мир");
        assert_eq!(normalize("один   два    три"), "один два три");
        assert_eq!(normalize("a.b,c;d"), "a b c d");
        assert_eq!(normalize("123-456"), "123 456");
        assert_eq!(normalize("!@#$%"), "");
    }

    #[test]
    fn normalize_digits() {
        assert_eq!(normalize("abc123"), "abc123");
        assert_eq!(normalize("123 456"), "123 456");
        assert_eq!(normalize("12.34"), "12 34");
    }

    #[test]
    fn levenshtein_empty() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "xyz"), 3);
    }

    #[test]
    fn levenshtein_equal() {
        assert_eq!(levenshtein("rust", "rust"), 0);
        assert_eq!(levenshtein("привет", "привет"), 0);
    }

    #[test]
    fn levenshtein_different() {
        assert_eq!(levenshtein("кот", "код"), 1);
        assert_eq!(levenshtein("abc", "def"), 3);
        assert_eq!(levenshtein("hello", "hallo"), 1);
        assert_eq!(levenshtein("sitting", "kitten"), 3);
    }

    #[test]
    fn match_text_exact() {
        let test = TestCfgTest::text_answer("правильный", vec![]);
        assert!(match_text("правильный", &test.to_cfg_test()));
        assert!(match_text("Правильный", &test.to_cfg_test()));
        assert!(match_text("правильный ", &test.to_cfg_test()));
        assert!(match_text("правилЬный", &test.to_cfg_test()));
    }

    #[test]
    fn match_text_with_accept() {
        let test = TestCfgTest::text_answer(
            "ответ",
            vec!["вариант".to_string(), "другой".to_string()],
        );
        assert!(match_text("вариант", &test.to_cfg_test()));
        assert!(match_text("Другой", &test.to_cfg_test()));
        assert!(match_text("ответ", &test.to_cfg_test()));
        assert!(!match_text("неверно", &test.to_cfg_test()));
    }

    #[test]
    fn match_text_levenshtein() {
        let test = TestCfgTest::text_answer("абсолютный", vec![]);
        assert!(match_text("абсолюный", &test.to_cfg_test()));
        assert!(match_text("абсолютнй", &test.to_cfg_test()));
        assert!(!match_text("абсолюнй", &test.to_cfg_test()));

        let test = TestCfgTest::text_answer("семь", vec![]);
        assert!(!match_text("сем", &test.to_cfg_test()));
    }

    #[test]
    fn match_text_empty_answer() {
        let test = TestCfgTest::text_answer("правильный", vec![]);
        assert!(!match_text("", &test.to_cfg_test()));
        assert!(!match_text(" ", &test.to_cfg_test()));
    }

    #[test]
    fn match_text_non_text_answer() {
        let test = TestCfgTest::single_answer();
        assert!(!match_text("что угодно", &test.to_cfg_test()));

        let test = TestCfgTest::multiple_answer();
        assert!(!match_text("что угодно", &test.to_cfg_test()));
    }

    #[test]
    fn match_text_fail() {
        let test = TestCfgTest::text_answer("правильный", vec![]);
        assert!(match_text("неправильно", &test.to_cfg_test())); // всегда false, тест упадёт
    }
}
