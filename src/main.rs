use crate::cli::cli::CliApp;
use crate::config::config::Config;
use anyhow::{Context, Result};

mod cli;
mod config;
mod dto;
mod types;
mod utils;

fn main() {
    if let Err(e) = run_app() {
        eprintln!("{:#}", e);
        std::process::exit(1);
    }
}

fn run_app() -> Result<()> {
    let cfg = Config::load().context("Ошибка загрузки конфига")?;
    let app = CliApp::new(&cfg);
    app.run().context("Ошибка запуска тестов")?;
    Ok(())
}
