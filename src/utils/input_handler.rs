use crate::utils::console_clear::flush_screen;
use crate::types::console_operation_type::ConsoleOperation;
use anyhow::Result;
use std::io;

pub fn ask_multiple_choice(max: usize) -> Result<(Vec<usize>, bool)> {
    loop {
        print!("Ваши ответы (1-{} через запятую): ", max);
        flush_screen()?;

        let (line, stop) = ask_line()?;
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
                    indices.sort();
                    return Ok((indices, false));
                }
                println!("Введите корректные номера от 1 до {} через запятую или !пропуск.", max);
            }
        }
    }
}

pub fn ask_single_choice(max: usize) -> Result<(i32, bool)> {
    loop {
        print!("Ваш ответ (1-{}): ", max);
        flush_screen()?;

        let (line, stop) = ask_line()?;
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
                println!("Введите число от 1 до {} или !пропуск.", max);
            }
        }
    }
}

pub fn ask_text_input() -> Result<(String, bool)> {
    loop {
        print!("Ваш ответ: ");
        flush_screen()?;

        let (line, stop) = ask_line()?;
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
                println!("Ответ не может быть пустым. Введите текст или !пропуск.");
            }
        }
    }
}

pub fn ask_line() -> Result<(String, bool)> {
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Ok((String::new(), true));
    }
    let trimmed = line.trim_end_matches(&['\r', '\n'][..]);

    match ConsoleOperation::from_input(trimmed) {
        ConsoleOperation::Quit => Ok((String::new(), true)),
        _ => Ok((trimmed.to_string(), false)),
    }
}
