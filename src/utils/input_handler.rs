use crate::types::console_operation_type::ConsoleOperation;
use anyhow::Result;
use std::io::{BufRead, BufReader, Write, stdin, stdout};

pub fn ask_line() -> Result<(String, bool)> {
    let mut input = BufReader::new(stdin());
    let mut output = stdout();
    ask_line_from(&mut input, &mut output)
}

pub fn ask_line_from<R: BufRead, W: Write>(input: &mut R, _: &mut W) -> Result<(String, bool)> {
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok((String::new(), true));
    }
    let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
    match ConsoleOperation::from_input(trimmed) {
        ConsoleOperation::Quit => Ok((String::new(), true)),
        _ => Ok((trimmed.to_string(), false)),
    }
}

pub fn ask_multiple_choice_from<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    max: usize,
) -> Result<(Vec<usize>, bool)> {
    loop {
        write!(output, "Ваши ответы (1-{} через запятую): ", max)?;
        output.flush()?;

        let (line, stop) = ask_line_from(input, output)?;
        if stop {
            return Ok((Vec::new(), true));
        }

        match ConsoleOperation::from_input(&line) {
            ConsoleOperation::Skip => return Ok((Vec::new(), false)),
            ConsoleOperation::Quit => return Ok((Vec::new(), true)),
            _ => {
                let line = line.trim();

                if let Ok(num) = line.parse::<usize>() {
                    if num >= 1 && num <= max {
                        return Ok((vec![num - 1], false));
                    }
                }

                let mut indices = Vec::new();
                let parts: Vec<&str> = line.split(',').collect();
                let mut valid = true;

                for part in parts {
                    let trimmed = part.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(num) = trimmed.parse::<usize>() {
                        if num >= 1 && num <= max {
                            if !indices.contains(&(num - 1)) {
                                indices.push(num - 1);
                            }
                        } else {
                            valid = false;
                            break;
                        }
                    } else {
                        valid = false;
                        break;
                    }
                }

                if valid && !indices.is_empty() {
                    return Ok((indices, false));
                }
                writeln!(
                    output,
                    "Введите корректные номера от 1 до {} через запятую или !пропуск.",
                    max
                )?;
                output.flush()?;
            }
        }
    }
}

pub fn ask_single_choice_from<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    max: usize,
) -> Result<(i32, bool)> {
    loop {
        write!(output, "Ваш ответ (1-{}): ", max)?;
        output.flush()?;

        let (line, stop) = ask_line_from(input, output)?;
        if stop {
            return Ok((0, true));
        }

        match ConsoleOperation::from_input(&line) {
            ConsoleOperation::Skip => return Ok((-1, false)),
            ConsoleOperation::Quit => return Ok((0, true)),
            _ => {
                if let Ok(num) = line.trim().parse::<usize>() {
                    if num >= 1 && num <= max {
                        return Ok((num as i32 - 1, false));
                    }
                }
                writeln!(output, "Введите число от 1 до {} или !пропуск.", max)?;
                output.flush()?;
            }
        }
    }
}

pub fn ask_text_input_from<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
) -> Result<(String, bool)> {
    loop {
        write!(output, "Ваш ответ: ")?;
        output.flush()?;

        let (line, stop) = ask_line_from(input, output)?;
        if stop {
            return Ok((String::new(), true));
        }

        match ConsoleOperation::from_input(&line) {
            ConsoleOperation::Skip => return Ok((String::new(), false)),
            ConsoleOperation::Quit => return Ok((String::new(), true)),
            _ => {
                let line = line.trim();
                if !line.is_empty() {
                    return Ok((line.to_string(), false));
                }
                writeln!(
                    output,
                    "Ответ не может быть пустым. Введите текст или !пропуск."
                )?;
                output.flush()?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn ask_line_from_returns_text() {
        let mut input = Cursor::new("hello\n".as_bytes());
        let mut output = Vec::new();
        let (line, stop) = ask_line_from(&mut input, &mut output).unwrap();
        assert_eq!(line, "hello");
        assert!(!stop);
    }

    #[test]
    fn ask_line_from_handles_quit() {
        let mut input = Cursor::new("!выход\n".as_bytes());
        let mut output = Vec::new();
        let (line, stop) = ask_line_from(&mut input, &mut output).unwrap();
        assert_eq!(line, "");
        assert!(stop);
    }

    #[test]
    fn ask_line_from_eof_returns_stop() {
        let mut input = Cursor::new("".as_bytes());
        let mut output = Vec::new();
        let (line, stop) = ask_line_from(&mut input, &mut output).unwrap();
        assert_eq!(line, "");
        assert!(stop);
    }

    #[test]
    fn ask_text_input_valid() {
        let mut input = Cursor::new("мой ответ\n".as_bytes());
        let mut output = Vec::new();
        let (text, stop) = ask_text_input_from(&mut input, &mut output).unwrap();
        assert_eq!(text, "мой ответ");
        assert!(!stop);
    }

    #[test]
    fn ask_text_input_empty_then_nonempty() {
        let mut input = Cursor::new("\nтекст\n".as_bytes());
        let mut output = Vec::new();
        let (text, stop) = ask_text_input_from(&mut input, &mut output).unwrap();
        assert_eq!(text, "текст");
        assert!(!stop);
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Ответ не может быть пустым"));
    }

    #[test]
    fn ask_text_input_skip() {
        let mut input = Cursor::new("!пропуск\n".as_bytes());
        let (text, stop) = ask_text_input_from(&mut input, &mut Vec::new()).unwrap();
        assert_eq!(text, "");
        assert!(!stop);
    }

    #[test]
    fn ask_text_input_quit() {
        let mut input = Cursor::new("!выход\n".as_bytes());
        let (text, stop) = ask_text_input_from(&mut input, &mut Vec::new()).unwrap();
        assert_eq!(text, "");
        assert!(stop);
    }

    #[test]
    fn ask_multiple_choice_single_number() {
        let mut input = Cursor::new("2\n".as_bytes());
        let mut output = Vec::new();
        let (indices, stop) = ask_multiple_choice_from(&mut input, &mut output, 3).unwrap();
        assert_eq!(indices, vec![1]);
        assert!(!stop);
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Ваши ответы (1-3 через запятую):"));
    }

    #[test]
    fn ask_multiple_choice_multiple_numbers() {
        let mut input = Cursor::new("1, 3, 2\n".as_bytes());
        let mut output = Vec::new();
        let (indices, stop) = ask_multiple_choice_from(&mut input, &mut output, 3).unwrap();
        assert_eq!(indices, vec![0, 2, 1]);
        assert!(!stop);
    }

    #[test]
    fn ask_multiple_choice_invalid_then_valid() {
        let mut input = Cursor::new("5\n2\n".as_bytes());
        let mut output = Vec::new();
        let (indices, stop) = ask_multiple_choice_from(&mut input, &mut output, 3).unwrap();
        assert_eq!(indices, vec![1]);
        assert!(!stop);
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Введите корректные номера"));
    }

    #[test]
    fn ask_multiple_choice_skip() {
        let mut input = Cursor::new("!пропуск\n".as_bytes());
        let mut output = Vec::new();
        let (indices, stop) = ask_multiple_choice_from(&mut input, &mut output, 3).unwrap();
        assert!(indices.is_empty());
        assert!(!stop);
    }

    #[test]
    fn ask_multiple_choice_quit() {
        let mut input = Cursor::new("!выход\n".as_bytes());
        let mut output = Vec::new();
        let (indices, stop) = ask_multiple_choice_from(&mut input, &mut output, 3).unwrap();
        assert!(indices.is_empty());
        assert!(stop);
    }

    #[test]
    fn ask_single_choice_valid() {
        let mut input = Cursor::new("3\n".as_bytes());
        let mut output = Vec::new();
        let (idx, stop) = ask_single_choice_from(&mut input, &mut output, 5).unwrap();
        assert_eq!(idx, 2);
        assert!(!stop);
    }

    #[test]
    fn ask_single_choice_invalid_then_valid() {
        let mut input = Cursor::new("0\n2\n".as_bytes());
        let mut output = Vec::new();
        let (idx, stop) = ask_single_choice_from(&mut input, &mut output, 3).unwrap();
        assert_eq!(idx, 1);
        assert!(!stop);
        let out = String::from_utf8(output).unwrap();
        assert!(out.contains("Введите число от 1 до 3"));
    }

    #[test]
    fn ask_single_choice_skip() {
        let mut input = Cursor::new("!пропуск\n".as_bytes());
        let (idx, stop) = ask_single_choice_from(&mut input, &mut Vec::new(), 3).unwrap();
        assert_eq!(idx, -1);
        assert!(!stop);
    }

    #[test]
    fn ask_single_choice_quit() {
        let mut input = Cursor::new("!выход\n".as_bytes());
        let (idx, stop) = ask_single_choice_from(&mut input, &mut Vec::new(), 3).unwrap();
        assert_eq!(idx, 0);
        assert!(stop);
    }
}
