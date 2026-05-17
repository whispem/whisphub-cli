use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::api;
use crate::config;
use crate::project;

const IGNORED_DIRS: &[&str] = &[
    "node_modules", "target", "dist", "build", ".git", ".svn",
    "__pycache__", ".venv", "venv", ".next", ".nuxt", ".cache",
    ".idea", ".vscode",
];

const IGNORED_FILES: &[&str] = &[".DS_Store", "Thumbs.db", ".env", ".env.local"];

const MAX_PUSH_SIZE_BYTES: u64 = 100 * 1024 * 1024;

pub async fn run(skip_confirm: bool) -> Result<()> {
    let auth = config::load()?
        .ok_or_else(|| anyhow!("not logged in. Run `whisphub login` to authenticate."))?;

    let cwd = std::env::current_dir()?;
    let project_cfg = project::load(&cwd)?.ok_or_else(|| {
        anyhow!("no .whisphub.toml found in current directory. Run `whisphub init` first.")
    })?;

    println!("{}", "Scanning files...".dimmed());
    let files = scan_files(&cwd)?;

    if files.is_empty() {
        return Err(anyhow!(
            "no files to push (current directory is empty or all ignored)"
        ));
    }

    let total_size: u64 = files.iter().map(|(_, size)| size).sum();

    if total_size > MAX_PUSH_SIZE_BYTES {
        return Err(anyhow!(
            "total size {} exceeds limit of {} MB",
            format_size(total_size),
            MAX_PUSH_SIZE_BYTES / 1024 / 1024
        ));
    }

    println!();
    println!(
        "Project: {} {}",
        project_cfg.slug.cyan().bold(),
        format!("(@{})", project_cfg.username).dimmed()
    );
    println!(
        "Files:   {} ({})",
        files.len().to_string().cyan().bold(),
        format_size(total_size).dimmed()
    );
    println!(
        "Ignored: {}",
        "node_modules/, target/, .git/, ...".dimmed()
    );
    println!();

    if !skip_confirm {
        let ok = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Push these files?")
            .default(true)
            .interact()?;
        if !ok {
            return Err(anyhow!("push cancelled."));
        }
    }

    println!("{}", "Creating archive...".dimmed());
    let zip_bytes = build_zip(&cwd, &files)?;

    println!("{}", "Uploading...".dimmed());
    let resp = api::upload_zip(
        &auth.token,
        &project_cfg.project_id,
        zip_bytes,
        &format!("{}.zip", project_cfg.slug),
    )
    .await?;

    let frontend = api::frontend_base();
    println!();
    println!(
        "{} Uploaded {} file{}",
        "✓".green().bold(),
        resp.imported_count.to_string().cyan().bold(),
        if resp.imported_count == 1 { "" } else { "s" }
    );
    println!(
        "{} Whispered to {}",
        "✓".green().bold(),
        format!(
            "{}/{}/{}",
            frontend, project_cfg.username, project_cfg.slug
        )
        .cyan()
        .underline()
    );

    Ok(())
}

fn scan_files(root: &Path) -> Result<Vec<(std::path::PathBuf, u64)>> {
    let mut result = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        if e.file_type().is_dir() && IGNORED_DIRS.contains(&name.as_ref()) {
            return false;
        }
        true
    }) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy();
        if IGNORED_FILES.contains(&name.as_ref()) {
            continue;
        }

        if name == ".whisphub.toml" {
            continue;
        }

        let rel = match entry.path().strip_prefix(root) {
            Ok(p) => p.to_path_buf(),
            Err(_) => continue,
        };

        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        result.push((rel, size));
    }

    Ok(result)
}

fn build_zip(root: &Path, files: &[(std::path::PathBuf, u64)]) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buffer);
        let mut zip = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        for (rel_path, _) in files {
            let abs_path = root.join(rel_path);
            let path_str = rel_path.to_string_lossy().replace('\\', "/");

            let content = std::fs::read(&abs_path)
                .map_err(|e| anyhow!("failed to read {}: {}", rel_path.display(), e))?;

            zip.start_file(path_str, options)?;
            zip.write_all(&content)?;
        }

        zip.finish()?;
    }

    Ok(buffer)
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
    }
}