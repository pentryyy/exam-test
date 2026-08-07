use crate::cli::cli::run;
use crate::config::config::Config;
use anyhow::{Context, Result};

mod config;
mod dto;
mod util;
mod cli;

fn main() -> Result<()> {
    let cfg = Config::load().context("Ошибка загрузки конфига")?;
    run(&cfg).context("Ошибка запуска тестов")?;
    Ok(())
}