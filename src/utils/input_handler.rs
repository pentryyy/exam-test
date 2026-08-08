use std::io;
use crate::utils::clr_operation::flush_screen;

pub fn wait_for_start() -> anyhow::Result<bool> {
    loop {
        let (line, stop) = ask_line()?;
        if stop {
            return Ok(false);
        }
        let line = line.trim();
        if line.is_empty() || line == "!старт" {
            return Ok(true);
        }
        if is_quit(line) {
            return Ok(false);
        }
        println!("Неизвестная команда. Введите !старт.");
    }
}

pub fn ask_multiple_choice(max: usize) -> anyhow::Result<(Vec<usize>, bool)> {
    loop {
        print!("Ваши ответы (1-{} через запятую): ", max);
        flush_screen()?;

        let (line, stop) = ask_line()?;
        if stop {
            return Ok((Vec::new(), true));
        }
        let line = line.trim();
        if is_skip(line) {
            return Ok((Vec::new(), false));
        }

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
        println!("Введите корректные номера от 1 до {} через запятую.", max);
    }
}

pub fn ask_single_choice(max: usize) -> anyhow::Result<(i32, bool)> {
    loop {
        print!("Ваш ответ (1-{}): ", max);
        flush_screen()?;

        let (line, stop) = ask_line()?;
        if stop {
            return Ok((0, true));
        }
        let line = line.trim();
        if is_skip(line) {
            return Ok((-1, false));
        }
        if let Ok(num) = line.parse::<usize>() {
            if num >= 1 && num <= max {
                return Ok((num as i32 - 1, false));
            }
        }
        println!("Введите число от 1 до {}.", max);
    }
}

pub fn ask_text_input() -> anyhow::Result<(String, bool)> {
    loop {
        print!("Ваш ответ: ");
        flush_screen()?;

        let (line, stop) = ask_line()?;
        if stop {
            return Ok((String::new(), true));
        }
        let line = line.trim();
        if is_skip(line) {
            return Ok((String::new(), false));
        }

        if !line.is_empty() {
            return Ok((line.to_string(), false));
        }

        println!("Ответ не может быть пустым. Введите текст или !пропуск.");
    }
}

pub fn ask_line() -> anyhow::Result<(String, bool)> {
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Ok((String::new(), true));
    }
    let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
    if is_quit(trimmed) {
        return Ok((String::new(), true));
    }
    Ok((trimmed.to_string(), false))
}

pub fn is_skip(s: &str) -> bool {
    let low = s.to_lowercase();
    low == "!пропуск" || low == "!skip"
}

pub fn is_restart(s: &str) -> bool {
    let low = s.to_lowercase();
    low == "!рестарт" || low == "!restart"
}

pub fn is_quit(s: &str) -> bool {
    let low = s.to_lowercase();
    low == "!выход" || low == "!quit" || low == "!exit"
}