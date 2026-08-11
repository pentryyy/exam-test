use std::io;
use std::io::Write;
use std::process::Command;

pub fn flush_screen() -> anyhow::Result<()> {
    io::stdout().flush()?;
    Ok(())
}

pub fn clear_screen() -> anyhow::Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "cls"]).status()?;
        Ok(())
    } else {
        print!("\x1B[2J\x1B[1;1H");
        flush_screen()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flush_screen_works() {
        assert!(flush_screen().is_ok());
    }

    #[test]
    fn clear_screen_works() {
        assert!(clear_screen().is_ok());
    }
}
