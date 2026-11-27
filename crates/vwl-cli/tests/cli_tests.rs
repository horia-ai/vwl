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
    // Initialize git repo in temp dir so GitInspector::find_repo_root finds it
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
