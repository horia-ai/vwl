use std::collections::HashMap;
use std::sync::Mutex;
use tempfile::tempdir;
use vwl_core::decision::{Decision, DecisionOutcome};
use vwl_core::github::{resolve_github_token, GithubClient, HttpTransport};
use vwl_core::ledger::LocalLedger;
use vwl_core::proof::{Proof, ProofType, Verdict};
use vwl_core::task::Task;
use vwl_core::work_unit::WorkUnit;
use vwl_core::Result;

type RecordedRequest = (String, String, Option<String>, Option<String>);

struct MockHttpTransport {
    pub responses: Mutex<HashMap<String, String>>,
    pub recorded_requests: Mutex<Vec<RecordedRequest>>,
}

impl MockHttpTransport {
    fn new() -> Self {
        Self {
            responses: Mutex::new(HashMap::new()),
            recorded_requests: Mutex::new(Vec::new()),
        }
    }

    fn set_response(&self, url_suffix: &str, body: &str) {
        self.responses
            .lock()
            .unwrap()
            .insert(url_suffix.to_string(), body.to_string());
    }
}

impl HttpTransport for MockHttpTransport {
    fn request(
        &self,
        method: &str,
        url: &str,
        token: Option<&str>,
        body: Option<&str>,
    ) -> Result<String> {
        self.recorded_requests.lock().unwrap().push((
            method.to_string(),
            url.to_string(),
            token.map(|s| s.to_string()),
            body.map(|s| s.to_string()),
        ));

        let responses = self.responses.lock().unwrap();
        for (suffix, resp) in responses.iter() {
            if url.ends_with(suffix) || url.contains(suffix) {
                return Ok(resp.clone());
            }
        }

        Err(vwl_core::VwlError::LedgerError(format!(
            "Mock 404 Not Found for URL: {}",
            url
        )))
    }
}

#[test]
fn test_token_resolution_priority() {
    // 1. Explicit token resolver logic
    let token = resolve_github_token();
    // In our test environment, ATRIFAN_GITHUB_PAT_TOKEN or GH_TOKEN is present
    if std::env::var("GH_TOKEN").is_ok()
        || std::env::var("GITHUB_TOKEN").is_ok()
        || std::env::var("ATRIFAN_GITHUB_PAT_TOKEN").is_ok()
    {
        assert!(token.is_some());
        assert!(!token.unwrap().trim().is_empty());
    }
}

#[test]
fn test_github_client_get_user_with_mock() {
    let mock = MockHttpTransport::new();
    mock.set_response(
        "/user",
        r#"{"login": "atrifan", "id": 1337, "name": "Alex Trifan"}"#,
    );

    let client = GithubClient::with_transport(Some("test_tok_123".to_string()), mock);
    let user = client.get_user().expect("get_user failed");

    assert_eq!(user.login, "atrifan");
    assert_eq!(user.id, 1337);
    assert_eq!(client.token(), Some("test_tok_123"));
}

#[test]
fn test_github_client_list_issues_and_get_issue_with_mock() {
    let mock = MockHttpTransport::new();
    mock.set_response(
        "/repos/horia-ai/vwl/issues?state=open",
        r#"[
            {
                "number": 55,
                "title": "feat(dogfooding): self-host and dogfood VWL",
                "state": "open",
                "html_url": "https://github.com/horia-ai/vwl/issues/55",
                "labels": [{"name": "type:experiment"}, {"name": "priority:p0"}],
                "body": "Dogfooding specification"
            }
        ]"#,
    );
    mock.set_response(
        "/repos/horia-ai/vwl/issues/55",
        r#"{
            "number": 55,
            "title": "feat(dogfooding): self-host and dogfood VWL",
            "state": "open",
            "html_url": "https://github.com/horia-ai/vwl/issues/55",
            "labels": [{"name": "type:experiment"}],
            "body": "Dogfooding specification"
        }"#,
    );

    let client = GithubClient::with_transport(Some("test_tok".to_string()), mock);
    let issues = client
        .list_issues("horia-ai", "vwl", Some("open"))
        .expect("list_issues failed");

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].number, 55);
    assert_eq!(issues[0].labels, vec!["type:experiment", "priority:p0"]);

    let issue = client
        .get_issue("horia-ai", "vwl", 55)
        .expect("get_issue failed");
    assert_eq!(issue.number, 55);
    assert_eq!(issue.title, "feat(dogfooding): self-host and dogfood VWL");
}

#[test]
fn test_github_client_add_comment_with_mock() {
    let mock = MockHttpTransport::new();
    mock.set_response(
        "/repos/horia-ai/vwl/issues/55/comments",
        r#"{
            "id": 999123,
            "body": "Proof Confirmed by Zero-Chat Verifier",
            "created_at": "2026-11-27T02:00:00Z"
        }"#,
    );

    let client = GithubClient::with_transport(Some("test_tok".to_string()), mock);
    let comment = client
        .add_issue_comment(
            "horia-ai",
            "vwl",
            55,
            "Proof Confirmed by Zero-Chat Verifier",
        )
        .expect("add_comment failed");

    assert_eq!(comment.id, 999123);
    assert!(comment.body.contains("Proof Confirmed"));
}

#[test]
fn test_github_create_issue_with_mock() {
    let mock = MockHttpTransport::new();
    mock.set_response(
        "/repos/horia-ai/vwl/issues",
        r#"{
            "number": 101,
            "title": "feat: follow-up spec update",
            "state": "open",
            "html_url": "https://github.com/horia-ai/vwl/issues/101",
            "labels": [{"name": "vwl-task"}],
            "body": "Falsifiable acceptance criteria..."
        }"#,
    );

    let client = GithubClient::with_transport(Some("test_tok".to_string()), mock);
    let issue = client
        .create_issue(
            "horia-ai",
            "vwl",
            "feat: follow-up spec update",
            Some("Falsifiable acceptance criteria..."),
            &["vwl-task"],
        )
        .expect("create_issue failed");

    assert_eq!(issue.number, 101);
    assert_eq!(issue.title, "feat: follow-up spec update");
    assert_eq!(issue.labels, vec!["vwl-task"]);
}

#[test]
fn test_github_sync_ledger_with_mock() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    let ledger = LocalLedger::new(repo_root).unwrap();

    let task = Task::new(
        "T-GH-001",
        "GitHub Sync Test",
        vec!["criteria 1".to_string()],
        "base-sha",
    )
    .unwrap();
    ledger.append_task(&task).unwrap();

    let wu = WorkUnit::new("T-GH-001", "cand-sha-001", "base-sha").unwrap();
    ledger.append_work_unit(&wu).unwrap();

    let proof = Proof::new(
        "cand-sha-001",
        ProofType::Work,
        Verdict::Pass,
        "cargo test",
        Some("All passed".to_string()),
    )
    .unwrap();
    ledger.append_proof(&proof).unwrap();

    let decision = Decision {
        subject: "cand-sha-001".to_string(),
        outcome: DecisionOutcome::Confirmed,
        reason: "All proofs passed".to_string(),
    };
    ledger.append_decision(&decision).unwrap();

    let mock = MockHttpTransport::new();
    mock.set_response(
        "/repos/horia-ai/vwl/issues",
        r#"[
            {
                "number": 1,
                "title": "WU-G0-001 — Bootstrap Rust workspace",
                "state": "open",
                "html_url": "https://github.com/horia-ai/vwl/issues/1",
                "labels": [{"name": "status:ready"}],
                "body": "Task details"
            }
        ]"#,
    );

    let client = GithubClient::with_transport(Some("test_tok".to_string()), mock);
    let summary = client
        .sync_ledger_to_github(repo_root, "horia-ai", "vwl")
        .expect("sync failed");

    assert_eq!(summary.tasks_count, 1);
    assert_eq!(summary.decisions_count, 1);
    assert!(summary.synced);
}

#[test]
fn test_live_github_api_whoami_with_env_token() {
    if let Ok(client) = GithubClient::from_env() {
        if let Ok(user) = client.get_user() {
            println!("Live GitHub API Authenticated as: {}", user.login);
            assert!(!user.login.is_empty());
        }
    }
}
