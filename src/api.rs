use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub fn api_base() -> String {
    std::env::var("WHISPHUB_API_URL").unwrap_or_else(|_| "https://whisphub.dev".to_string())
}

pub fn frontend_base() -> String {
    std::env::var("WHISPHUB_FRONTEND_URL").unwrap_or_else(|_| "https://whisphub.dev".to_string())
}

#[derive(Debug, Deserialize)]
pub struct CliAuthInitResponse {
    pub code: String,
    pub device_code: String,
    pub verification_uri: String,
    #[allow(dead_code)]
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize)]
struct PollPayload<'a> {
    device_code: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct CliAuthPollResponse {
    pub status: String,
    pub token: Option<String>,
    pub user: Option<CliAuthUser>,
}

#[derive(Debug, Deserialize)]
pub struct CliAuthUser {
    #[allow(dead_code)]
    pub id: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct MeResponse {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
struct CreateProjectPayload<'a> {
    title: &'a str,
    slug: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    tagline: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub slug: String,
    #[allow(dead_code)]
    pub title: String,
    pub author_username: String,
}

#[derive(Debug, Deserialize)]
pub struct ZipUploadResponse {
    pub imported_count: usize,
}

pub async fn init_device_flow() -> Result<CliAuthInitResponse> {
    let client = Client::new();
    let url = format!("{}/api/cli/auth/init", api_base());
    let res = client.post(&url).send().await?;

    if !res.status().is_success() {
        return Err(anyhow!("init failed: HTTP {}", res.status()));
    }

    Ok(res.json::<CliAuthInitResponse>().await?)
}

pub async fn poll_device_flow(device_code: &str) -> Result<CliAuthPollResponse> {
    let client = Client::new();
    let url = format!("{}/api/cli/auth/poll", api_base());
    let payload = PollPayload { device_code };

    let res = client.post(&url).json(&payload).send().await?;

    if !res.status().is_success() {
        let status = res.status();
        let body: serde_json::Value = res.json().await.unwrap_or_default();
        let msg = body
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error")
            .to_string();
        return Err(anyhow!("poll failed: {} ({})", msg, status));
    }

    Ok(res.json::<CliAuthPollResponse>().await?)
}

pub async fn me(token: &str) -> Result<MeResponse> {
    let client = Client::new();
    let url = format!("{}/api/auth/me", api_base());
    let res = client.get(&url).bearer_auth(token).send().await?;

    if !res.status().is_success() {
        return Err(anyhow!(
            "auth check failed: HTTP {} (token may be expired)",
            res.status()
        ));
    }

    Ok(res.json::<MeResponse>().await?)
}

pub async fn create_project(
    token: &str,
    title: &str,
    slug: &str,
    tagline: Option<&str>,
) -> Result<ProjectSummary> {
    let client = Client::new();
    let url = format!("{}/api/projects", api_base());
    let payload = CreateProjectPayload { title, slug, tagline };

    let res = client
        .post(&url)
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await?;

    let status = res.status();
    let body_text = res.text().await.context("failed to read response body")?;

    if !status.is_success() {
        let parsed: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
        let msg = parsed
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or(&body_text)
            .to_string();
        return Err(anyhow!("create project failed: {}", msg));
    }

    serde_json::from_str::<ProjectSummary>(&body_text)
        .with_context(|| format!("failed to parse project response. Raw body: {}", body_text))
}

pub async fn upload_zip(
    token: &str,
    project_id: &str,
    zip_bytes: Vec<u8>,
    filename: &str,
) -> Result<ZipUploadResponse> {
    let client = Client::new();
    let url = format!("{}/api/projects/{}/files/zip", api_base(), project_id);

    let part = reqwest::multipart::Part::bytes(zip_bytes)
        .file_name(filename.to_string())
        .mime_str("application/zip")?;
    let form = reqwest::multipart::Form::new().part("file", part);

    let res = client
        .post(&url)
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await?;

    let status = res.status();
    let body_text = res.text().await.context("failed to read response body")?;

    if !status.is_success() {
        let parsed: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
        let msg = parsed
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or(&body_text)
            .to_string();
        return Err(anyhow!("upload failed: {} ({})", msg, status));
    }

    serde_json::from_str::<ZipUploadResponse>(&body_text)
        .with_context(|| format!("failed to parse upload response. Raw body: {}", body_text))
}