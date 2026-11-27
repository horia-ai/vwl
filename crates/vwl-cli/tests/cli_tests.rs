use std::process::Command;

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
