use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_help_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .arg("--help")
        .output()
        .expect("Failed to execute vwl --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Virtual Work Ledger"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("task"));
    assert!(stdout.contains("replay"));
    assert!(stdout.contains("worker"));
    assert!(stdout.contains("skill"));
    assert!(stdout.contains("node"));
    assert!(stdout.contains("market"));
    assert!(stdout.contains("economics"));
}

#[test]
fn test_cli_status_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .arg("status")
        .output()
        .expect("Failed to execute vwl status");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Virtual Work Ledger (VWL) Status:"));
    assert!(stdout.contains("Repository Root:"));
    assert!(stdout.contains("HEAD SHA:"));
}

#[test]
fn test_cli_report_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .arg("report")
        .output()
        .expect("Failed to execute vwl report");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("VIRTUAL WORK LEDGER (VWL) — CONTRIBUTORS & COST REPORT"));
    assert!(stdout.contains("CONTRIBUTORS & EFFECTIVENESS"));
    assert!(stdout.contains("COST & COMPENSATION ALLOCATION"));
}

#[test]
fn test_cli_skill_list_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["skill", "list"])
        .output()
        .expect("Failed to execute vwl skill list");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available VWL Protocol & Gate Skills:"));
    assert!(stdout.contains("tdd-mutation-testing"));
    assert!(stdout.contains("bounded-verifier"));
    assert!(stdout.contains("economic-conservation"));
    assert!(stdout.contains("dex-liquidity-rules"));
}

#[test]
fn test_cli_skill_install_command() {
    let temp = tempdir().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to init git");

    let output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "skill",
            "install",
            "tdd-mutation-testing",
            "--harness",
            "all",
        ])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute vwl skill install");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Successfully installed skill 'tdd-mutation-testing' for harness 'all'")
    );

    assert!(temp
        .path()
        .join(".vwl/skills/tdd-mutation-testing/SKILL.md")
        .exists());
    assert!(temp
        .path()
        .join(".claude/skills/tdd-mutation-testing/SKILL.md")
        .exists());
    assert!(temp
        .path()
        .join(".gemini/skills/tdd-mutation-testing/SKILL.md")
        .exists());
    assert!(temp.path().join(".github/copilot-instructions.md").exists());

    let copilot_content =
        fs::read_to_string(temp.path().join(".github/copilot-instructions.md")).unwrap();
    assert!(copilot_content.contains("## Skill: tdd-mutation-testing"));
}

#[test]
fn test_cli_node_identity_lifecycle() {
    let temp = tempdir().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to init git");

    // 1. vwl node init
    let init_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["node", "init", "--role", "developer", "--name", "Test Node"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute node init");
    assert!(init_output.status.success());
    let init_str = String::from_utf8_lossy(&init_output.stdout);
    assert!(init_str.contains("Initialized Node Identity (Gate 1):"));
    assert!(init_str.contains("Node ID:     node-ed25519-"));

    // 2. vwl node show
    let show_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["node", "show"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute node show");
    assert!(show_output.status.success());
    let show_str = String::from_utf8_lossy(&show_output.stdout);
    assert!(show_str.contains("Active VWL Node Credential:"));
    assert!(show_str.contains("[VALID]"));

    // 3. vwl node sign
    let sign_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["node", "sign", "evidence-payload-abc"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute node sign");
    assert!(sign_output.status.success());
    let sig = String::from_utf8_lossy(&sign_output.stdout)
        .trim()
        .to_string();
    assert_eq!(sig.len(), 128); // 64 bytes in hex

    // 4. Extract pubkey from credential.yaml and verify
    let cred_content =
        fs::read_to_string(temp.path().join(".vwl/identity/credential.yaml")).unwrap();
    let pubkey_line = cred_content
        .lines()
        .find(|l| l.contains("public_key:"))
        .unwrap();
    let pubkey = pubkey_line.split(':').nth(1).unwrap().trim();

    let verify_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["node", "verify", pubkey, "evidence-payload-abc", &sig])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute node verify");
    assert!(verify_output.status.success());
    let verify_str = String::from_utf8_lossy(&verify_output.stdout);
    assert!(verify_str.contains("Signature VALID"));
}

#[test]
fn test_cli_market_commands() {
    let temp = tempdir().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to init git");

    // Write .vwl/project.yaml
    let vwl_dir = temp.path().join(".vwl");
    fs::create_dir_all(&vwl_dir).unwrap();
    let manifest_yaml = r#"
name: "test-project"
repository: "https://github.com/horia-ai/test-project"
description: "Test Decentralized Project"
topics: ["vwl-project"]
roles:
  - id: "ROLE-BACKEND"
    title: "Rust Backend Engineer"
    skills: ["tdd-mutation-testing"]
    budget_work: 200000
    status: "Open"
"#;
    fs::write(vwl_dir.join("project.yaml"), manifest_yaml).unwrap();

    // 1. vwl market list
    let list_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["market", "list"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute market list");
    assert!(list_output.status.success());
    let list_str = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_str.contains("VWL Project Discovery (Gate 2):"));
    assert!(list_str.contains("ROLE-BACKEND"));

    // 2. Initialize node identity
    Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "node",
            "init",
            "--role",
            "developer",
            "--name",
            "Gemini Node",
        ])
        .current_dir(temp.path())
        .output()
        .expect("Failed to init node");

    // 3. vwl market offer
    let offer_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["market", "offer", "ROLE-BACKEND", "--budget", "190000"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute market offer");
    assert!(offer_output.status.success());
    let offer_str = String::from_utf8_lossy(&offer_output.stdout);
    assert!(offer_str.contains("Successfully Submitted Cryptographic Offer (Gate 2):"));
    assert!(offer_str.contains("Ask Budget:   190000 WORK"));
}

#[test]
fn test_cli_economics_commands() {
    let temp = tempdir().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to init git");

    // 1. vwl economics deposit
    let dep_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["economics", "deposit", "horia-ai/vwl", "1000"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to deposit funds");
    assert!(dep_output.status.success());
    let dep_str = String::from_utf8_lossy(&dep_output.stdout);
    assert!(dep_str.contains("Minted 1000000 WORK"));

    // 2. vwl economics engage
    let eng_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "economics",
            "engage",
            "horia-ai/vwl",
            "node-gemini-dev",
            "250000",
            "--vesting-secs",
            "3600",
        ])
        .current_dir(temp.path())
        .output()
        .expect("Failed to create engagement");
    assert!(eng_output.status.success());
    let eng_str = String::from_utf8_lossy(&eng_output.stdout);
    assert!(eng_str.contains("Successfully Created Engagement (Gate 5):"));
    assert!(eng_str.contains("Allocated:     250000 WORK"));

    // 3. vwl economics verify
    let ver_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["economics", "verify"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to verify conservation invariant");
    assert!(ver_output.status.success());
    let ver_str = String::from_utf8_lossy(&ver_output.stdout);
    assert!(ver_str.contains("Economic Conservation Invariant Strictly Verified: PASSED"));

    // 4. vwl economics show
    let show_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["economics", "show"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to show economics");
    assert!(show_output.status.success());
    let show_str = String::from_utf8_lossy(&show_output.stdout);
    assert!(show_str.contains("VWL Local Economic State Machine (Gate 5):"));
    assert!(show_str.contains("Reserved 250000"));
}

#[test]
fn test_cli_worker_run_and_tui_render() {
    let temp = tempdir().unwrap();
    // 1. Initialize git repo and create a base commit
    Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "test@vwl.network"])
        .current_dir(temp.path())
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "VWL Tester"])
        .current_dir(temp.path())
        .output()
        .expect("git config name");
    fs::write(temp.path().join("README.md"), "# Test Repo\n").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(temp.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(temp.path())
        .output()
        .expect("git commit");

    let head_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp.path())
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head_output.stdout)
        .trim()
        .to_string();

    // 2. Add a Task
    let task_yaml = format!(
        r#"id: "TASK-RUNNER-001"
objective: "Implement autonomous task test"
acceptance:
  - "Unit tests pass cleanly"
base: "{}"
"#,
        head_sha
    );
    let task_path = temp.path().join("task_runner.yaml");
    fs::write(&task_path, task_yaml).unwrap();

    let add_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "add", task_path.to_str().unwrap()])
        .current_dir(temp.path())
        .output()
        .expect("vwl task add");
    assert!(add_output.status.success());

    // 3. Execute vwl worker run --max-tasks 1
    let run_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["worker", "run", "--harness", "gemini", "--max-tasks", "1"])
        .current_dir(temp.path())
        .output()
        .expect("vwl worker run");
    assert!(run_output.status.success());
    let run_str = String::from_utf8_lossy(&run_output.stdout);
    assert!(run_str.contains("Picked up Task: TASK-RUNNER-001"));
    assert!(run_str.contains("Work Unit produced"));
    assert!(run_str.contains("Proof recorded: Type Work, Verdict Pass"));
    assert!(run_str.contains("Deterministic Decision: Confirmed"));
    assert!(run_str.contains("Successfully completed cycle for task TASK-RUNNER-001"));

    // 4. Verify vwl tui headless render
    let tui_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["tui", "--render-view", "0"])
        .current_dir(temp.path())
        .output()
        .expect("vwl tui");
    assert!(tui_output.status.success());
    let tui_str = String::from_utf8_lossy(&tui_output.stdout);
    assert!(tui_str.contains("Headless TUI render for view 0 successful"));
}

#[test]
fn test_cli_gh_whoami_and_issues() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["gh", "whoami", "--offline"])
        .env("GH_TOKEN", "mock_gh_token_for_test")
        .output()
        .expect("vwl gh whoami");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GitHub API Authentication (Offline / Env Check):"));
    assert!(stdout.contains("Token Source: GH_TOKEN"));

    let issues_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["gh", "issues", "--offline"])
        .output()
        .expect("vwl gh issues");

    assert!(issues_output.status.success());
    let issues_str = String::from_utf8_lossy(&issues_output.stdout);
    assert!(issues_str.contains("GitHub Issues (Offline Projection):"));
}

#[test]
fn test_cli_harness_list_and_start() {
    let temp = tempdir().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .expect("git init");

    let list_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["harness", "list"])
        .current_dir(temp.path())
        .output()
        .expect("vwl harness list");

    assert!(list_output.status.success());
    let list_str = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_str.contains("Supported VWL Agent Harnesses:"));
    assert!(list_str.contains("gemini"));
    assert!(list_str.contains("opencode"));
    assert!(list_str.contains("claude"));
    assert!(list_str.contains("copilot"));

    let start_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["harness", "start", "--type", "gemini", "--dry-run"])
        .current_dir(temp.path())
        .output()
        .expect("vwl harness start");

    assert!(start_output.status.success());
    let start_str = String::from_utf8_lossy(&start_output.stdout);
    assert!(start_str.contains("Preflighting agent harness: gemini"));
    assert!(start_str.contains("Harness Ready"));
}

#[test]
fn test_cli_mcp_list_and_status() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["mcp", "list"])
        .output()
        .expect("vwl mcp list");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("VWL Model Context Protocol (MCP) Integrations:"));
    assert!(stdout.contains("@vwl/stripe-mcp"));
    assert!(stdout.contains("github-mcp"));
    assert!(stdout.contains("stripe_create_payment_intent"));

    let status_output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["mcp", "status"])
        .output()
        .expect("vwl mcp status");

    assert!(status_output.status.success());
    let status_str = String::from_utf8_lossy(&status_output.stdout);
    assert!(status_str.contains("MCP Runtime Status:"));
}

#[test]
fn test_cli_wizard_summary_retro_banner() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["wizard", "--summary"])
        .output()
        .expect("vwl wizard");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verifies retro robot ASCII geometry from icon.svg
    assert!(stdout.contains("(o)"));
    assert!(stdout.contains("(•) (•)"));
    assert!(stdout.contains("[█]   [█]   [█]"));
    assert!(stdout.contains("VIRTUAL WORK LEDGER"));
    assert!(stdout.contains("Interactive Protocol Console"));
    assert!(stdout.contains("[1] 🪙 Wallet & Economics"));
    assert!(stdout.contains("[2] 📋 Project & Market Roles"));
    assert!(stdout.contains("[3] 🤖 Agent Harness & Worker"));
    assert!(stdout.contains("[4] 🐙 GitHub API Sync"));
    assert!(stdout.contains("[5] 🔌 MCP Integrations"));
    assert!(stdout.contains("[6] 🖥️ Launch TUI"));
}

#[test]
fn test_cli_frontier_dag_and_claims_workflow() {
    let temp = tempfile::tempdir().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "tester@vwl.network"])
        .current_dir(temp.path())
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "VWL Tester"])
        .current_dir(temp.path())
        .output()
        .expect("git config name");
    fs::write(temp.path().join("README.md"), "# Frontier Test Repo\n").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(temp.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(temp.path())
        .output()
        .expect("git commit");

    let head_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp.path())
        .output()
        .expect("git rev-parse");
    let head_sha = String::from_utf8_lossy(&head_output.stdout)
        .trim()
        .to_string();

    // Ingest TASK-A and TASK-B
    let task_a_yaml = format!(
        r#"id: "TASK-A"
objective: "Build core protocol component"
acceptance: ["Component compiles"]
base: "{}"
"#,
        head_sha
    );
    let path_a = temp.path().join("task_a.yaml");
    fs::write(&path_a, task_a_yaml).unwrap();
    let add_a = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "add", path_a.to_str().unwrap()])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(add_a.status.success());

    let task_b_yaml = format!(
        r#"id: "TASK-B"
objective: "Build consumer CLI tool"
acceptance: ["CLI responds to help"]
base: "{}"
"#,
        head_sha
    );
    let path_b = temp.path().join("task_b.yaml");
    fs::write(&path_b, task_b_yaml).unwrap();
    let add_b = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "add", path_b.to_str().unwrap()])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(add_b.status.success());

    // Record dependency TASK-B requires TASK-A
    let dep_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "add-dep", "TASK-A", "TASK-B", "--kind", "requires"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(dep_out.status.success());

    // Check why-blocked on TASK-B
    let why_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "why-blocked", "TASK-B"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(why_out.status.success());
    let why_str = String::from_utf8_lossy(&why_out.stdout);
    assert!(why_str.contains("Requires unconfirmed task 'TASK-A'"));

    // Check deps on TASK-B
    let deps_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "deps", "TASK-B"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(deps_out.status.success());
    let deps_str = String::from_utf8_lossy(&deps_out.stdout);
    assert!(deps_str.contains("UPSTREAM:   TASK-A (requires)"));

    // Check frontier: TASK-A must be on frontier, TASK-B must NOT
    let frontier_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["frontier"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(frontier_out.status.success());
    let front_str = String::from_utf8_lossy(&frontier_out.stdout);
    assert!(front_str.contains("TASK-A"));
    assert!(!front_str.contains("TASK-B"));
    assert!(front_str.contains("[READY - UNCLAIMED]"));

    // Claim TASK-A
    let claim_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["claim", "TASK-A", "--actor", "gemini-agent"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(claim_out.status.success());

    // Frontier reflects CLAIMED status
    let frontier_claimed_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["frontier"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let front_claimed_str = String::from_utf8_lossy(&frontier_claimed_out.stdout);
    assert!(front_claimed_str.contains("[CLAIMED by gemini-agent]"));

    // Intent graph
    let graph_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["graph", "intent"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(graph_out.status.success());
    let graph_str = String::from_utf8_lossy(&graph_out.stdout);
    assert!(graph_str.contains("Task TASK-A"));
    assert!(graph_str.contains("Task TASK-B"));
    assert!(graph_str.contains("REQUIRES upstream TASK-A"));

    // Submit WorkUnit for TASK-A
    let submit_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["submit", "TASK-A"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(submit_out.status.success());
    let submit_str = String::from_utf8_lossy(&submit_out.stdout);
    assert!(submit_str.contains("WorkUnit submitted successfully for task TASK-A"));
    assert!(submit_str.contains("Dependency Snapshot: sha256:"));

    // Release claim
    let release_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["release", "TASK-A"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(release_out.status.success());
}
