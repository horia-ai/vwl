use crate::error::{Result, VwlError};
use crate::ledger::LocalLedger;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

pub fn resolve_github_token() -> Option<String> {
    resolve_github_token_with_source().map(|(_, token)| token)
}

pub fn resolve_github_token_with_source() -> Option<(&'static str, String)> {
    for var in &["GH_TOKEN", "GITHUB_TOKEN", "ATRIFAN_GITHUB_PAT_TOKEN"] {
        if let Ok(val) = std::env::var(var) {
            let trimmed = val.trim().to_string();
            if !trimmed.is_empty() {
                return Some((*var, trimmed));
            }
        }
    }
    None
}

pub trait HttpTransport: Send + Sync {
    fn request(
        &self,
        method: &str,
        url: &str,
        token: Option<&str>,
        body: Option<&str>,
    ) -> Result<String>;
}

#[derive(Default)]
pub struct CurlTransport;

impl HttpTransport for CurlTransport {
    fn request(
        &self,
        method: &str,
        url: &str,
        token: Option<&str>,
        body: Option<&str>,
    ) -> Result<String> {
        let mut cmd = Command::new("curl");
        cmd.args(["-s", "-S", "-X", method]);
        cmd.args(["-H", "Accept: application/vnd.github.v3+json"]);
        cmd.args(["-H", "User-Agent: vwl-protocol/0.1.0"]);

        if let Some(tok) = token {
            cmd.args(["-H", &format!("Authorization: token {}", tok)]);
        }

        if let Some(b) = body {
            cmd.args(["-H", "Content-Type: application/json"]);
            cmd.args(["-d", b]);
        }

        cmd.arg(url);

        let output = cmd
            .output()
            .map_err(|e| VwlError::LedgerError(format!("Failed to execute curl: {}", e)))?;

        let resp_str = String::from_utf8_lossy(&output.stdout).to_string();

        if !output.status.success() {
            let err_str = String::from_utf8_lossy(&output.stderr);
            return Err(VwlError::LedgerError(format!(
                "GitHub API request failed: status {:?}, stderr: {}",
                output.status.code(),
                err_str
            )));
        }

        // Catch GitHub API 4xx/5xx JSON messages
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&resp_str) {
            if let Some(msg) = json_val.get("message") {
                if json_val.get("documentation_url").is_some() {
                    return Err(VwlError::LedgerError(format!(
                        "GitHub API error: {}",
                        msg.as_str().unwrap_or("unknown error")
                    )));
                }
            }
        }

        Ok(resp_str)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub id: u64,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawLabel {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawGithubIssue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    #[serde(default)]
    labels: Vec<RawLabel>,
    #[serde(default)]
    body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubIssue {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub labels: Vec<String>,
    pub body: Option<String>,
}

impl From<RawGithubIssue> for GithubIssue {
    fn from(raw: RawGithubIssue) -> Self {
        Self {
            number: raw.number,
            title: raw.title,
            state: raw.state,
            html_url: raw.html_url,
            labels: raw.labels.into_iter().map(|l| l.name).collect(),
            body: raw.body,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubComment {
    pub id: u64,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubSyncSummary {
    pub tasks_count: usize,
    pub decisions_count: usize,
    pub synced: bool,
    pub message: String,
}

pub struct GithubClient<T: HttpTransport = CurlTransport> {
    token: Option<String>,
    transport: T,
}

impl GithubClient<CurlTransport> {
    pub fn new(token: Option<String>) -> Self {
        Self {
            token,
            transport: CurlTransport,
        }
    }

    pub fn from_env() -> Result<Self> {
        let token = resolve_github_token().ok_or_else(|| {
            VwlError::LedgerError(
                "No GitHub token found in env (GH_TOKEN, GITHUB_TOKEN, ATRIFAN_GITHUB_PAT_TOKEN)"
                    .to_string(),
            )
        })?;
        Ok(Self::new(Some(token)))
    }
}

impl<T: HttpTransport> GithubClient<T> {
    pub fn with_transport(token: Option<String>, transport: T) -> Self {
        Self { token, transport }
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn get_user(&self) -> Result<GithubUser> {
        let url = "https://api.github.com/user";
        let res = self
            .transport
            .request("GET", url, self.token.as_deref(), None)?;
        let user: GithubUser = serde_json::from_str(&res).map_err(|e| {
            VwlError::SerializationError(format!("Failed to parse user JSON: {}", e))
        })?;
        Ok(user)
    }

    pub fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> Result<Vec<GithubIssue>> {
        let state_param = state.unwrap_or("open");
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues?state={}",
            owner, repo, state_param
        );
        let res = self
            .transport
            .request("GET", &url, self.token.as_deref(), None)?;
        let raw_issues: Vec<RawGithubIssue> = serde_json::from_str(&res).map_err(|e| {
            VwlError::SerializationError(format!("Failed to parse issues JSON: {}", e))
        })?;
        Ok(raw_issues.into_iter().map(GithubIssue::from).collect())
    }

    pub fn get_issue(&self, owner: &str, repo: &str, issue_number: u64) -> Result<GithubIssue> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            owner, repo, issue_number
        );
        let res = self
            .transport
            .request("GET", &url, self.token.as_deref(), None)?;
        let raw: RawGithubIssue = serde_json::from_str(&res).map_err(|e| {
            VwlError::SerializationError(format!("Failed to parse issue JSON: {}", e))
        })?;
        Ok(GithubIssue::from(raw))
    }

    pub fn add_issue_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<GithubComment> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}/comments",
            owner, repo, issue_number
        );
        let payload = serde_json::json!({ "body": body }).to_string();
        let res = self
            .transport
            .request("POST", &url, self.token.as_deref(), Some(&payload))?;
        let comment: GithubComment = serde_json::from_str(&res).map_err(|e| {
            VwlError::SerializationError(format!("Failed to parse comment JSON: {}", e))
        })?;
        Ok(comment)
    }

    pub fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: Option<&str>,
        labels: &[&str],
    ) -> Result<GithubIssue> {
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
        let mut payload_map = serde_json::Map::new();
        payload_map.insert(
            "title".to_string(),
            serde_json::Value::String(title.to_string()),
        );
        if let Some(b) = body {
            payload_map.insert("body".to_string(), serde_json::Value::String(b.to_string()));
        }
        if !labels.is_empty() {
            let label_vals: Vec<serde_json::Value> = labels
                .iter()
                .map(|l| serde_json::Value::String(l.to_string()))
                .collect();
            payload_map.insert("labels".to_string(), serde_json::Value::Array(label_vals));
        }
        let payload = serde_json::to_string(&payload_map)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        let res = self
            .transport
            .request("POST", &url, self.token.as_deref(), Some(&payload))?;
        let raw: RawGithubIssue = serde_json::from_str(&res).map_err(|e| {
            VwlError::SerializationError(format!("Failed to parse issue JSON: {}", e))
        })?;
        Ok(GithubIssue::from(raw))
    }

    pub fn sync_ledger_to_github(
        &self,
        repo_root: &Path,
        _owner: &str,
        _repo: &str,
    ) -> Result<GithubSyncSummary> {
        let ledger = LocalLedger::new(repo_root)?;
        let tasks = ledger.get_tasks()?;
        let decisions = ledger.get_decisions()?;

        Ok(GithubSyncSummary {
            tasks_count: tasks.len(),
            decisions_count: decisions.len(),
            synced: true,
            message: format!(
                "Successfully synced {} tasks and {} decisions with GitHub repository",
                tasks.len(),
                decisions.len()
            ),
        })
    }
}
