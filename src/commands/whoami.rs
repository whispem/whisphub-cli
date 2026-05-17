use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::api;
use crate::config;

pub async fn run() -> Result<()> {
    let auth = config::load()?
        .ok_or_else(|| anyhow!("not logged in. Run `whisphub login` to authenticate."))?;

    match api::me(&auth.token).await {
        Ok(me) => {
            println!(
                "Logged in as {} ({})",
                format!("@{}", me.username).cyan().bold(),
                me.email.dimmed()
            );
            Ok(())
        }
        Err(_) => Err(anyhow!(
            "your session has expired. Run `whisphub login` again."
        )),
    }
}