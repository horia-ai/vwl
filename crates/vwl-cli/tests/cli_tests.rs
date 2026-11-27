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
