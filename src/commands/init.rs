use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};

use crate::api;
use crate::config;
use crate::project::{self, ProjectConfig};

pub async fn run(
    title_arg: Option<String>,
    slug_arg: Option<String>,
    tagline_arg: Option<String>,
) -> Result<()> {
    let auth = config::load()?
        .ok_or_else(|| anyhow!("not logged in. Run `whisphub login` to authenticate."))?;

    let cwd = std::env::current_dir()?;

    if project::load(&cwd)?.is_some() {
        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("A .whisphub.toml already exists. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            return Err(anyhow!("init cancelled."));
        }
    }

    let title = match title_arg {
        Some(t) => t,
        None => Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Project title")
            .interact_text()?,
    };

    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(anyhow!("title cannot be empty"));
    }

    let suggested = project::slugify(&title);
    let slug = match slug_arg {
        Some(s) => s,
        None => Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("URL slug")
            .default(suggested.clone())
            .interact_text()?,
    };

    let slug = slug.trim().to_string();
    if slug.is_empty() {
        return Err(anyhow!("slug cannot be empty"));
    }

    let tagline = match tagline_arg {
        Some(t) => Some(t),
        None => {
            let input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Tagline (optional)")
                .allow_empty(true)
                .interact_text()?;
            if input.trim().is_empty() {
                None
            } else {
                Some(input)
            }
        }
    };

    println!("{}", "Creating project...".dimmed());
    let project_summary =
        api::create_project(&auth.token, &title, &slug, tagline.as_deref()).await?;

    project::save(
        &cwd,
        &ProjectConfig {
            project_id: project_summary.id.clone(),
            username: project_summary.author_username.clone(),
            slug: project_summary.slug.clone(),
        },
    )?;

    let frontend = api::frontend_base();
    println!();
    println!(
        "{} Created {}",
        "✓".green().bold(),
        format!(
            "{}/{}/{}",
            frontend, project_summary.author_username, project_summary.slug
        )
        .cyan()
        .underline()
    );
    println!(
        "{} Wrote {}",
        "✓".green().bold(),
        ".whisphub.toml".dimmed()
    );
    println!();
    println!(
        "{}",
        "Next: add some files, then run `whisphub push` to upload.".dimmed()
    );

    Ok(())
}