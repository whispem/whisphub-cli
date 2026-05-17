use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project_id: String,
    pub username: String,
    pub slug: String,
}

const CONFIG_FILENAME: &str = ".whisphub.toml";

pub fn config_path(dir: &Path) -> PathBuf {
    dir.join(CONFIG_FILENAME)
}

pub fn save(dir: &Path, cfg: &ProjectConfig) -> Result<()> {
    let path = config_path(dir);
    let content = toml::to_string_pretty(cfg).context("failed to serialize project config")?;
    let header = "# WhispHub project config — do not edit manually.\n\
                  # Created by `whisphub init`, used by `whisphub push`.\n\n";
    let full = format!("{}{}", header, content);
    std::fs::write(&path, full)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn load(dir: &Path) -> Result<Option<ProjectConfig>> {
    let path = config_path(dir);
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let cfg: ProjectConfig = toml::from_str(&raw)
        .with_context(|| format!("{} is malformed", path.display()))?;
    Ok(Some(cfg))
}

#[allow(dead_code)]
pub fn suggest_slug_from_dir(dir: &Path) -> Result<String> {
    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("could not determine current directory name"))?;
    Ok(slugify(name))
}

pub fn slugify(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut prev_dash = false;
    for c in lower.chars() {
        if c.is_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}