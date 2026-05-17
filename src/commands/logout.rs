use anyhow::Result;
use colored::Colorize;

use crate::config;

pub async fn run() -> Result<()> {
    let was_logged_in = config::load()?.is_some();

    config::clear()?;

    if was_logged_in {
        println!("{} Logged out.", "✓".green().bold());
    } else {
        println!("{}", "Not logged in. Nothing to do.".dimmed());
    }

    Ok(())
}