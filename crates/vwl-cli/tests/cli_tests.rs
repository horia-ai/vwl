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
    assert!(stdout.contains("repo-bootstrap-initializer"));
    assert!(stdout.contains("validator-wiring"));
    assert!(stdout.contains("validator-test-integrity"));
    assert!(stdout.contains("validator-targeted-mutation"));
    assert!(stdout.contains("validator-adversarial-skeptic"));
    assert!(stdout.contains("validator-shadow-smoke"));
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

#[test]
fn test_cli_verify_plan_and_run() {
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
    fs::write(temp.path().join("README.md"), "# Verify Test\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(temp.path())
        .output()
        .expect("git commit");

    // Test vwl verify plan
    let plan_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["verify", "plan", "cand_candidate_001"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(plan_out.status.success());
    let plan_str = String::from_utf8_lossy(&plan_out.stdout);
    assert!(plan_str.contains("VWL Verification Plan for Subject: cand_candidate_001"));
    assert!(plan_str.contains("[REQUIRED] compile"));
    assert!(plan_str.contains("[REQUIRED] unit"));
    assert!(plan_str.contains("[REQUIRED] wiring"));
    assert!(plan_str.contains("[REQUIRED] test-integrity"));

    // Test vwl verify run
    let run_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "verify",
            "run",
            "cand_candidate_001",
            "--verifier",
            "gemini",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(run_out.status.success());
    let run_str = String::from_utf8_lossy(&run_out.stdout);
    assert!(run_str.contains("Executing Verification Factory on WorkUnit 'cand_candidate_001'"));
    assert!(run_str.contains("Verification Decision: Confirmed"));
}

#[test]
fn test_cli_repometer_and_outcome_workflow() {
    let temp = tempfile::tempdir().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "repometer@vwl.network"])
        .current_dir(temp.path())
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "Repometer Tester"])
        .current_dir(temp.path())
        .output()
        .expect("git config name");
    fs::write(temp.path().join("README.md"), "# Repometer CLI Test\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "init repometer test"])
        .current_dir(temp.path())
        .output()
        .expect("git commit");

    // Ingest task
    let task_file = temp.path().join("task.yaml");
    fs::write(
        &task_file,
        "id: TASK-REPOMETER-01\nobjective: Repometer test\nacceptance: [pass]\nbase: c0\n",
    )
    .unwrap();
    let task_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "add", task_file.to_str().unwrap()])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(task_out.status.success());

    // Submit WorkUnit
    let submit_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "submit",
            "TASK-REPOMETER-01",
            "--candidate",
            "cand_repometer_01",
            "--base",
            "c0",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(submit_out.status.success());

    // Verify Run to produce decision
    let verify_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["verify", "run", "cand_repometer_01"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(verify_out.status.success());

    // 1. Test 'vwl repometer'
    let repometer_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["repometer"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(repometer_out.status.success());
    let repometer_str = String::from_utf8_lossy(&repometer_out.stdout);
    assert!(repometer_str.contains("VWL REPOMETER (Evidence & Lineage Coverage)"));
    assert!(repometer_str.contains("Canonical Tree:"));
    assert!(repometer_str.contains("Ledger-Attributed:"));
    assert!(repometer_str.contains("Independently Verified:"));
    assert!(repometer_str.contains("Overall Confidence:"));

    // 2. Test 'vwl repometer --explain'
    let repometer_explain_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["repometer", "--explain"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(repometer_explain_out.status.success());
    let explain_str = String::from_utf8_lossy(&repometer_explain_out.stdout);
    assert!(explain_str.contains("VWL Repometer Evidence Coverage"));
    assert!(explain_str.contains("Methodology: vwl-v2.2-lineage-and-evidence-coverage"));

    // 3. Test 'vwl repometer --json'
    let repometer_json_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["repometer", "--json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(repometer_json_out.status.success());
    let json_str = String::from_utf8_lossy(&repometer_json_out.stdout);
    assert!(json_str.contains("\"methodology\": \"vwl-v2.2-lineage-and-evidence-coverage\""));

    // 4. Test 'vwl outcome add'
    let outcome_add_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "outcome",
            "add",
            "O-CLI-REGRESS-1",
            "cand_repometer_01",
            "--type",
            "regression",
            "--severity",
            "high",
            "--evidence",
            "Memory leak found under load test",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(outcome_add_out.status.success());
    let outcome_add_str = String::from_utf8_lossy(&outcome_add_out.stdout);
    assert!(outcome_add_str.contains("Successfully recorded outcome 'O-CLI-REGRESS-1'"));

    // 5. Test 'vwl outcome list'
    let outcome_list_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["outcome", "list"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(outcome_list_out.status.success());
    let outcome_list_str = String::from_utf8_lossy(&outcome_list_out.stdout);
    assert!(outcome_list_str.contains("O-CLI-REGRESS-1"));
    assert!(outcome_list_str.contains("regression"));
    assert!(outcome_list_str.contains("cand_repometer_01"));

    // 6. Test 'vwl outcome impact'
    let outcome_impact_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["outcome", "impact", "O-CLI-REGRESS-1"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(outcome_impact_out.status.success());
    let outcome_impact_str = String::from_utf8_lossy(&outcome_impact_out.stdout);
    assert!(outcome_impact_str.contains("Impact Assessment for Outcome 'O-CLI-REGRESS-1'"));
}

#[test]
fn test_cli_v23_incident_and_sla_commands() {
    let temp = tempdir().unwrap();

    // 1. Initialize git repo
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // 2. Test 'vwl sla service'
    let sla_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["sla", "service", "payments", "--window", "30d"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(sla_out.status.success());
    let sla_str = String::from_utf8_lossy(&sla_out.stdout);
    assert!(sla_str.contains("SERVICE SLA / SLO: payments"));
    assert!(sla_str.contains("Availability:"));
    assert!(sla_str.contains("Window:                 30d"));

    // 3. Record an outcome to attach incident to
    let outcome_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "outcome",
            "add",
            "O-INC-CLI-1",
            "cand_root_wu",
            "--type",
            "regression",
            "--severity",
            "high",
            "--evidence",
            "Latency spike over 2000ms",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(outcome_out.status.success());

    // 4. Test 'vwl incident show'
    let show_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["incident", "show", "O-INC-CLI-1"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(show_out.status.success());
    let show_str = String::from_utf8_lossy(&show_out.stdout);
    assert!(show_str.contains("INCIDENT O-INC-CLI-1"));
    assert!(show_str.contains("Root WorkUnit:"));

    // 5. Test 'vwl incident contain'
    let contain_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "incident",
            "contain",
            "O-INC-CLI-1",
            "--strategy",
            "rollback",
            "--target",
            "release-v1.42.0",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(contain_out.status.success());
    let contain_str = String::from_utf8_lossy(&contain_out.stdout);
    assert!(contain_str.contains("Successfully contained incident 'O-INC-CLI-1'"));
    assert!(contain_str.contains("Rollback"));
    assert!(contain_str.contains("Service restored"));

    // 6. Test 'vwl incident metrics'
    let metrics_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["incident", "metrics", "O-INC-CLI-1"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(metrics_out.status.success());
    let metrics_str = String::from_utf8_lossy(&metrics_out.stdout);
    assert!(metrics_str.contains("INCIDENT METRICS O-INC-CLI-1"));
    assert!(metrics_str.contains("SRT (Service):"));

    // 7. Test 'vwl impact frontier'
    let frontier_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["impact", "frontier", "O-INC-CLI-1"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(frontier_out.status.success());
    let frontier_str = String::from_utf8_lossy(&frontier_out.stdout);
    assert!(frontier_str.contains("Immediate Affected Frontier (F1) for Outcome 'O-INC-CLI-1'"));

    // 8. Test 'vwl proof stale'
    let proof_stale_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["proof", "stale", "--because", "cand_root_wu"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(proof_stale_out.status.success());
    let proof_stale_str = String::from_utf8_lossy(&proof_stale_out.stdout);
    assert!(proof_stale_str.contains("Stale Proofs because of repair 'cand_root_wu'"));

    // 9. Test 'vwl revalidate frontier'
    let revalidate_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["revalidate", "frontier", "O-INC-CLI-1"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(revalidate_out.status.success());
    let reval_str = String::from_utf8_lossy(&revalidate_out.stdout);
    assert!(reval_str.contains("Revalidating immediate affected frontier"));

    // 10. Test 'vwl repair start'
    let repair_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["repair", "start", "cand_root_wu"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(repair_out.status.success());
    let repair_str = String::from_utf8_lossy(&repair_out.stdout);
    assert!(repair_str.contains("Started repair task for WorkUnit 'cand_root_wu'"));
    assert!(repair_str.contains("Erected repair barrier"));
}

#[test]
fn test_cli_spec_writer_and_role_nodes() {
    let temp = tempdir().unwrap();
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "tester@vwl.network"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "VWL Tester"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    fs::write(temp.path().join("README.md"), "# Role Nodes Test\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let head_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let head_sha = String::from_utf8_lossy(&head_output.stdout)
        .trim()
        .to_string();

    // 1. Spec-Writer Node: Review & Ingest with Skeptic Gate
    let valid_task_yaml = format!(
        r#"id: "T-SPEC-CLI-1"
objective: "Implement deterministic CLI command"
acceptance: ["cargo test --test cli_tests"]
base: "{}"
"#,
        head_sha
    );
    let valid_task_path = temp.path().join("task_valid.yaml");
    fs::write(&valid_task_path, &valid_task_yaml).unwrap();

    // vwl task review valid task
    let rev_valid = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "review", valid_task_path.to_str().unwrap()])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(rev_valid.status.success());
    let rev_valid_str = String::from_utf8_lossy(&rev_valid.stdout);
    assert!(rev_valid_str.contains("VERDICT: PASSED"));

    // vwl task review --json
    let rev_json = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "task",
            "review",
            valid_task_path.to_str().unwrap(),
            "--json",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(rev_json.status.success());
    let rev_json_str = String::from_utf8_lossy(&rev_json.stdout);
    assert!(rev_json_str.contains("\"passed\": true"));

    // Skeptic rejection of subjective criteria and conversational cue
    let bad_task_yaml = format!(
        r#"id: "T-SPEC-CLI-2"
objective: "Make the code clean as discussed in our previous chat"
acceptance: ["clean code that works well"]
base: "{}"
"#,
        head_sha
    );
    let bad_task_path = temp.path().join("task_bad.yaml");
    fs::write(&bad_task_path, &bad_task_yaml).unwrap();

    let rev_bad = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "review", bad_task_path.to_str().unwrap()])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!rev_bad.status.success());
    let rev_bad_str = String::from_utf8_lossy(&rev_bad.stdout);
    assert!(rev_bad_str.contains("VERDICT: FAILED"));
    assert!(rev_bad_str.contains("clean code"));
    assert!(rev_bad_str.contains("as discussed"));

    // vwl task add --skeptic on bad task -> rejected
    let add_bad = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "add", bad_task_path.to_str().unwrap(), "--skeptic"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!add_bad.status.success());
    let add_bad_err = String::from_utf8_lossy(&add_bad.stderr);
    assert!(add_bad_err.contains("rejected by Skeptic Review Gate"));

    // vwl task add --skeptic on valid task -> admitted
    let add_valid = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "task",
            "add",
            valid_task_path.to_str().unwrap(),
            "--skeptic",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(add_valid.status.success());
    let add_valid_str = String::from_utf8_lossy(&add_valid.stdout);
    assert!(add_valid_str.contains("Skeptic Review Gate: PASSED"));

    // 2. Developer Node: Bounded Execution (WU-G0-010)
    let dev_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["worker", "developer", "T-SPEC-CLI-1", "--harness", "gemini"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(dev_out.status.success());
    let dev_str = String::from_utf8_lossy(&dev_out.stdout);
    assert!(dev_str.contains("ZERO CONVERSATIONAL HISTORY"));
    assert!(dev_str.contains("Candidate WorkUnit produced"));

    // 3. Verifier Node: Independent Verification Factory Gate (WU-G0-011)
    let prefix = format!(
        "cand_t_spec_cli_1_{}",
        head_sha.chars().take(8).collect::<String>()
    );
    let val_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["worker", "verifier", &prefix, "--harness", "gemini"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(val_out.status.success());
    let val_str = String::from_utf8_lossy(&val_out.stdout);
    assert!(val_str.contains("IMPLEMENTER CHAT LOGS STRICTLY BARRED"));
    assert!(val_str.contains("Candidate Isolation"));
    assert!(val_str.contains("Structured Proof emitted"));
    assert!(val_str.contains("Decision Engine Outcome"));

    // 3b. Specialist Validator Node: Wiring Proof
    let val_wiring = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args([
            "worker",
            "verifier",
            &prefix,
            "--harness",
            "gemini",
            "--specialist",
            "wiring",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(val_wiring.status.success());
    let val_wiring_str = String::from_utf8_lossy(&val_wiring.stdout);
    assert!(val_wiring_str.contains("specialist='wiring'"));
    assert!(val_wiring_str.contains("gate='wiring'"));

    // 4. Role-based Autonomous Runner (Role Developer then Role Verifier)
    let task_3_yaml = format!(
        r#"id: "T-SPEC-CLI-3"
objective: "Implement role-based runner test"
acceptance: ["cargo test"]
base: "{}"
"#,
        head_sha
    );
    let task_3_path = temp.path().join("task_3.yaml");
    fs::write(&task_3_path, &task_3_yaml).unwrap();
    let add_3 = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["task", "add", task_3_path.to_str().unwrap(), "--skeptic"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(add_3.status.success());

    // Run as role: developer
    let run_dev = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["worker", "run", "--role", "developer", "--max-tasks", "1"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(run_dev.status.success());
    let run_dev_str = String::from_utf8_lossy(&run_dev.stdout);
    assert!(run_dev_str.contains("Role: developer"));
    assert!(run_dev_str.contains("Handing off to Validator"));

    // Run as role: verifier
    let run_val = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["worker", "run", "--role", "verifier", "--max-tasks", "1"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(run_val.status.success());
    let run_val_str = String::from_utf8_lossy(&run_val.stdout);
    assert!(run_val_str.contains("Role: verifier"));
    assert!(run_val_str.contains("Completed validation for candidate"));
}

#[test]
fn test_cli_repo_init_and_precommit_gates() {
    let temp = tempdir().unwrap();
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "init@vwl.network"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "VWL Init"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Create minimal package.json and tsconfig.json to simulate a project repo
    let pkg_json = r#"{
  "name": "target-app",
  "scripts": {
    "test": "echo 'running test suite'"
  }
}"#;
    fs::write(temp.path().join("package.json"), pkg_json).unwrap();
    fs::write(temp.path().join("tsconfig.json"), "{}\n").unwrap();

    // 1. Run vwl init
    let init_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["init"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(init_out.status.success());
    let init_str = String::from_utf8_lossy(&init_out.stdout);
    assert!(init_str.contains("VWL REPOSITORY INITIALIZATION COMPLETE"));
    assert!(init_str.contains("typescript"));
    assert!(init_str.contains(".vwl/gates.yaml"));
    assert!(init_str.contains("Installed at .git/hooks/pre-commit"));

    // Check files created
    assert!(temp.path().join(".vwl/gates.yaml").exists());
    assert!(temp.path().join(".vwl/project.yaml").exists());
    assert!(temp.path().join(".git/hooks/pre-commit").exists());

    // 2. Check vwl hook status
    let hook_status = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["hook", "status"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(hook_status.status.success());
    let hook_str = String::from_utf8_lossy(&hook_status.stdout);
    assert!(hook_str.contains("INSTALLED & ACTIVE"));

    // 3. Check vwl verify gates
    let gates_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["verify", "gates"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(gates_out.status.success());
    let gates_str = String::from_utf8_lossy(&gates_out.stdout);
    assert!(gates_str.contains("VWL CONFIGURED VALIDATOR GATES"));
    assert!(gates_str.contains("node-tests"));

    // 4. Run vwl verify precommit (with an echo gate)
    // Customise .vwl/gates.yaml with a deterministic echo gate
    let custom_gates = r#"version: "1.0"
project: "target-app"
languages: ["typescript"]
gates:
  - name: "fast-precommit"
    plane: "deterministic"
    command: "echo 'precommit checks pass'"
    precommit: true
    timeout_secs: 10
    description: "Fast gate"
"#;
    fs::write(temp.path().join(".vwl/gates.yaml"), custom_gates).unwrap();

    let precommit_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["verify", "precommit"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(precommit_out.status.success());
    let precommit_str = String::from_utf8_lossy(&precommit_out.stdout);
    assert!(precommit_str.contains("VWL PRE-COMMIT VALIDATOR GATES"));
    assert!(precommit_str.contains("fast-precommit"));
    assert!(precommit_str.contains("PASSED"));

    // 5. Test vwl hook uninstall and reinstall
    let uninst_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["hook", "uninstall"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(uninst_out.status.success());
    assert!(!temp.path().join(".git/hooks/pre-commit").exists());

    let reinst_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["hook", "install"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(reinst_out.status.success());
    assert!(temp.path().join(".git/hooks/pre-commit").exists());
}

#[test]
fn test_cli_bootstrap_workflow() {
    let temp = tempdir().unwrap();
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Create an initial commit
    let dummy_file = temp.path().join("README.md");
    fs::write(&dummy_file, "# Test Repo\n").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Execute vwl bootstrap
    let boot_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["bootstrap", "--harness", "opencode", "--commit"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(boot_out.status.success());
    let boot_str = String::from_utf8_lossy(&boot_out.stdout);
    assert!(boot_str.contains("VWL REPOSITORY BOOTSTRAP INITIALIZER"));
    assert!(boot_str.contains("[1/5] Discovered project language stack"));
    assert!(boot_str.contains("[2/5] Validating static pre-commit validator gates"));
    assert!(boot_str.contains("[3/5] Installing Git pre-commit hook"));
    assert!(boot_str.contains("[4/5] Injecting agent skills for harness 'opencode'"));
    assert!(boot_str.contains("[5/5] Checking node cryptographic identity"));
    assert!(boot_str.contains("VWL REPOSITORY BOOTSTRAP COMPLETE"));

    // Verify filesystem state
    assert!(temp.path().join(".vwl/gates.yaml").exists());
    assert!(temp.path().join(".vwl/project.yaml").exists());
    assert!(temp.path().join(".vwl/identity/credential.yaml").exists());
    assert!(temp.path().join(".git/hooks/pre-commit").exists());
    assert!(temp.path().join(".opencode/skills/repo-bootstrap-initializer/SKILL.md").exists());
    assert!(temp.path().join(".opencode/skills/validator-wiring/SKILL.md").exists());

    // Verify git commit
    let log_out = Command::new("git")
        .args(["log", "-n", "1", "--oneline"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let log_str = String::from_utf8_lossy(&log_out.stdout);
    assert!(log_str.contains("chore(vwl): bootstrap VWL protocol gates"));
}

#[test]
fn test_cli_harness_auto_bootstrap() {
    let temp = tempdir().unwrap();
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let dummy_file = temp.path().join("README.md");
    fs::write(&dummy_file, "# Test Repo\n").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // In an uninitialized repo, run vwl harness start --type opencode --dry-run
    let harness_out = Command::new(env!("CARGO_BIN_EXE_vwl"))
        .args(["harness", "start", "--type", "opencode", "--dry-run"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(harness_out.status.success());
    let harness_str = String::from_utf8_lossy(&harness_out.stdout);
    assert!(harness_str.contains("[VWL Harness] Repository has VWL or is starting fresh"));
    assert!(harness_str.contains("VWL REPOSITORY BOOTSTRAP COMPLETE"));
    assert!(harness_str.contains("Preflighting agent harness: opencode"));
    assert!(harness_str.contains("Harness Ready"));

    // Verify that gates and hooks were generated
    assert!(temp.path().join(".vwl/gates.yaml").exists());
    assert!(temp.path().join(".git/hooks/pre-commit").exists());
}
