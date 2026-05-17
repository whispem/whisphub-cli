use anyhow::{anyhow, Result};
use colored::Colorize;
use std::time::Duration;

use crate::api;
use crate::config::{self, AuthConfig};

const MAX_POLL_DURATION_SECS: u64 = 600;

pub async fn run() -> Result<()> {
    println!("{}", "Initializing WhispHub login...".dimmed());

    if let Ok(Some(existing)) = config::load() {
        if api::me(&existing.token).await.is_ok() {
            println!(
                "{} Already logged in as {}.",
                "✓".green(),
                format!("@{}", existing.username).cyan().bold()
            );
            println!("  Run {} to switch accounts.", "whisphub logout".dimmed());
            return Ok(());
        }
    }

    let init = api::init_device_flow().await?;

    println!();
    println!("{}", "Open this URL in your browser to authorize:".bold());
    println!("  {}", init.verification_uri.cyan().underline());
    println!();
    println!("{}", "Your verification code:".bold());
    println!("  {}", init.code.cyan().bold());
    println!();

    let _ = webbrowser::open(&init.verification_uri);

    println!("{}", "Waiting for authorization...".dimmed());

    let poll_interval = Duration::from_secs(init.interval.max(1));
    let max_iterations = MAX_POLL_DURATION_SECS / init.interval.max(1);

    for _ in 0..max_iterations {
        tokio::time::sleep(poll_interval).await;

        match api::poll_device_flow(&init.device_code).await {
            Ok(resp) => match resp.status.as_str() {
                "pending" => continue,
                "expired" => {
                    return Err(anyhow!(
                        "code expired before approval. Run `whisphub login` again."
                    ));
                }
                "authorized" => {
                    let token = resp.token.ok_or_else(|| anyhow!("server returned no token"))?;
                    let user = resp.user.ok_or_else(|| anyhow!("server returned no user"))?;

                    config::save(&AuthConfig {
                        token,
                        username: user.username.clone(),
                    })?;

                    println!();
                    println!(
                        "{} Logged in as {}.",
                        "✓".green().bold(),
                        format!("@{}", user.username).cyan().bold()
                    );
                    return Ok(());
                }
                other => return Err(anyhow!("unexpected status: {}", other)),
            },
            Err(e) => {
                eprintln!("{} {}", "warning:".yellow(), e);
                continue;
            }
        }
    }

    Err(anyhow!(
        "timed out waiting for authorization. Run `whisphub login` again."
    ))
}