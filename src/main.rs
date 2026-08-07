use crate::cli::cli::run;
use crate::config::config::Config;
use anyhow::{Context, Result};

mod config;
mod dto;
mod util;
mod cli;

fn main() {
    if let Err(e) = run_app() {
        eprintln!("{:#}", e);
        std::process::exit(1);
    }
}

fn run_app() -> Result<()> {
    let cfg = Config::load().context("Ошибка загрузки конфига")?;
    run(&cfg).context("Ошибка запуска тестов")?;
    Ok(())
}