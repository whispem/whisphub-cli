use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: String,
    pub username: String,
}

#[cfg(not(target_os = "windows"))]
fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow!("could not determine home directory"))?;
    Ok(home.join(".config").join("whisphub"))
}

#[cfg(target_os = "windows")]
fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow!("could not determine config directory"))?;
    Ok(base.join("whisphub"))
}

fn auth_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("auth.json"))
}

pub fn save(auth: &AuthConfig) -> Result<()> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;

    let path = auth_path()?;
    let json = serde_json::to_string_pretty(auth)?;
    std::fs::write(&path, json)
        .with_context(|| format!("failed to write {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

pub fn load() -> Result<Option<AuthConfig>> {
    let path = auth_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let auth: AuthConfig = serde_json::from_str(&raw)
        .with_context(|| "auth file is corrupted, try `whisphub login` again")?;
    Ok(Some(auth))
}

pub fn clear() -> Result<()> {
    let path = auth_path()?;
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("failed to remove {}", path.display()))?;
    }
    Ok(())
}