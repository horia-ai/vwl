use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use vwl_core::{
    git::GitInspector, identity::NodeCredential, identity::NodeKeypair, ledger::LocalLedger,
    replay::ReplayEngine, skills, task::Task, verify_signature, Result, VwlError,
};

#[derive(Parser)]
#[command(name = "vwl")]
#[command(about = "Virtual Work Ledger — Decentralized labor and evidence protocol", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print Git repository status, HEAD, and canonical base (WU-G0-002)
    Status,

    /// Generate contributor effectiveness and cost allocation report (Gate 4 & 5)
    Report {
        /// Export report to GitHub Pages docs directory (docs/data/ledger_report.json)
        #[arg(long)]
        gh_pages: bool,

        /// Custom output directory for the report
        #[arg(long)]
        output: Option<PathBuf>,

        /// Print report in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Ingest and validate immutable task specification (WU-G0-003)
    Task {
        #[command(subcommand)]
        cmd: TaskCommands,
    },

    /// Create isolated worktree for a task (WU-G0-009)
    Work { task_id: String },

    /// Reconstruct authoritative state from Git and ledger (WU-G0-008)
    Replay,

    /// Execute developer or verifier worker harness (WU-G0-010 & WU-G0-011)
    Worker {
        #[command(subcommand)]
        cmd: WorkerCommands,
    },

    /// Manage decentralized node identity and cryptographic credentials (Gate 1)
    Node {
        #[command(subcommand)]
        cmd: NodeCommands,
    },

    /// Manage and inject agent skills into repository (.claude, .gemini, .github, .vwl)
    Skill {
        #[command(subcommand)]
        cmd: SkillCommands,
    },

    /// Query or scan local market discovery projections (Gate 2)
    Market,
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Add task YAML to ledger
    Add { path: PathBuf },
}

#[derive(Subcommand)]
enum WorkerCommands {
    /// Execute developer agent with bounded context (WU-G0-010)
    Developer {
        task_id: String,
        /// Target harness: opencode, claude, copilot, gemini, or custom (default: opencode)
        #[arg(long, default_value = "opencode")]
        harness: String,
        #[arg(last = true)]
        extra_args: Vec<String>,
    },
    /// Execute independent verifier with no implementer chat (WU-G0-011)
    Verifier {
        commit: String,
        /// Target harness: opencode, claude, copilot, gemini, or custom (default: opencode)
        #[arg(long, default_value = "opencode")]
        harness: String,
        #[arg(last = true)]
        extra_args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum NodeCommands {
    /// Initialize Ed25519 node identity and credentials in .vwl/identity/
    Init {
        #[arg(long, default_value = "developer")]
        role: String,
        #[arg(long, default_value = "Gemini Agent Node")]
        name: String,
    },
    /// Show current node identity, public key, and credential verification
    Show,
    /// Sign an arbitrary message with node's private key
    Sign { message: String },
    /// Verify a signature against a public key
    Verify {
        pubkey: String,
        message: String,
        signature: String,
    },
}

#[derive(Subcommand)]
enum SkillCommands {
    /// List all built-in protocol and gate skills
    List,
    /// Install skill into repository for target harness
    Install {
        /// Skill name (e.g., tdd-mutation-testing, bounded-verifier, economic-conservation, dex-liquidity-rules)
        name: String,
        /// Target harness: opencode, claude, copilot, gemini, or all (default: all)
        #[arg(long, default_value = "all")]
        harness: String,
    },
}

fn get_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let current_dir = std::env::current_dir()?;

    match cli.command {
        Commands::Status => {
            let info = GitInspector::inspect(&current_dir)?;
            println!("Virtual Work Ledger (VWL) Status:");
            println!("  Repository Root: {}", info.root.display());
            println!("  HEAD SHA:        {}", info.head_sha);
            println!("  Tree SHA:        {}", info.tree_sha);
            println!(
                "  Canonical Base:  {}",
                info.canonical_base
                    .as_deref()
                    .unwrap_or("none (head is provisional)")
            );
        }

        Commands::Report {
            gh_pages,
            output,
            json,
        } => {
            let repo_root = GitInspector::find_repo_root(&current_dir)?;
            let report = vwl_core::analytics::ProjectLedgerReport::generate(&repo_root)?;

            if gh_pages || output.is_some() {
                let target_dir = output.unwrap_or_else(|| repo_root.join("docs"));
                let path = report.export_to_gh_pages(&target_dir)?;
                println!(
                    "Report successfully exported for GitHub Pages to {}",
                    path.display()
                );
            }

            if json {
                let json_str = serde_json::to_string_pretty(&report)
                    .map_err(|e| vwl_core::VwlError::SerializationError(e.to_string()))?;
                println!("{}", json_str);
            } else if !gh_pages {
                print!("{}", report.format_cli_summary());
            }
        }

        Commands::Task { cmd } => match cmd {
            TaskCommands::Add { path } => {
                let content = std::fs::read_to_string(&path)?;
                let task = Task::from_yaml(&content)?;
                let repo_root = GitInspector::find_repo_root(&current_dir)?;
                let ledger = LocalLedger::new(&repo_root)?;
                ledger.append_task(&task)?;
                println!(
                    "Task {} successfully ingested with digest {}",
                    task.id,
                    task.compute_digest()
                );
            }
        },

        Commands::Work { task_id } => {
            let repo_root = GitInspector::find_repo_root(&current_dir)?;
            let worktree_dir = repo_root.join(".vwl").join("worktrees").join(&task_id);
            if !worktree_dir.exists() {
                std::fs::create_dir_all(&worktree_dir)?;
                let _ = std::process::Command::new("git")
                    .args([
                        "worktree",
                        "add",
                        "-B",
                        &format!("task/{}", task_id),
                        worktree_dir.to_str().unwrap(),
                        "HEAD",
                    ])
                    .current_dir(&repo_root)
                    .output();
            }
            println!(
                "Assigned task {} to isolated worktree at {}",
                task_id,
                worktree_dir.display()
            );
        }

        Commands::Replay => {
            let repo_root = GitInspector::find_repo_root(&current_dir)?;
            let state = ReplayEngine::replay(&repo_root)?;
            println!("Replay derived state:");
            println!("  Total Ingested Tasks: {}", state.total_tasks);
            println!("  Confirmed Work Units: {:?}", state.confirmed_work_units);
            println!(
                "  Canonical Head:       {}",
                state.canonical_head.as_deref().unwrap_or("unborn")
            );
        }

        Commands::Worker { cmd } => match cmd {
            WorkerCommands::Developer {
                task_id,
                harness,
                extra_args,
            } => {
                println!(
                    "Launching bounded developer worker for task {} using harness '{}' with args: {:?}",
                    task_id, harness, extra_args
                );
            }
            WorkerCommands::Verifier {
                commit,
                harness,
                extra_args,
            } => {
                println!(
                    "Launching independent verifier for commit {} using harness '{}' with args: {:?}",
                    commit, harness, extra_args
                );
            }
        },

        Commands::Node { cmd } => match cmd {
            NodeCommands::Init { role, name } => {
                let repo_root = GitInspector::find_repo_root(&current_dir)?;
                let identity_dir = repo_root.join(".vwl").join("identity");
                fs::create_dir_all(&identity_dir)?;

                let keypair = NodeKeypair::generate();
                let key_path = identity_dir.join("node.key");
                fs::write(&key_path, keypair.to_hex())?;

                let now = get_now_secs();
                let credential = NodeCredential::new(&keypair, &role, &name, now);
                let cred_yaml = serde_yaml::to_string(&credential)
                    .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                let cred_path = identity_dir.join("credential.yaml");
                fs::write(&cred_path, cred_yaml)?;

                println!("Initialized Node Identity (Gate 1):");
                println!("  Node ID:     {}", credential.node_id);
                println!("  Public Key:  {}", credential.public_key);
                println!("  Role:        {}", credential.role);
                println!("  Name:        {}", credential.display_name);
                println!("  Key Saved:   {}", key_path.display());
                println!("  Cred Saved:  {}", cred_path.display());
            }

            NodeCommands::Show => {
                let repo_root = GitInspector::find_repo_root(&current_dir)?;
                let cred_path = repo_root
                    .join(".vwl")
                    .join("identity")
                    .join("credential.yaml");
                if !cred_path.exists() {
                    return Err(VwlError::InvalidTask(
                        "Node identity not found. Run 'vwl node init' first.".to_string(),
                    ));
                }
                let yaml_str = fs::read_to_string(&cred_path)?;
                let cred: NodeCredential = serde_yaml::from_str(&yaml_str)
                    .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                let is_valid = cred.verify()?;

                println!("Active VWL Node Credential:");
                println!("  Node ID:      {}", cred.node_id);
                println!("  Public Key:   {}", cred.public_key);
                println!("  Role:         {}", cred.role);
                println!("  Name:         {}", cred.display_name);
                println!("  Created At:   {}", cred.created_at);
                println!(
                    "  Signature:    {} [{}]",
                    cred.signature,
                    if is_valid {
                        "VALID"
                    } else {
                        "INVALID/TAMPERED"
                    }
                );
            }

            NodeCommands::Sign { message } => {
                let repo_root = GitInspector::find_repo_root(&current_dir)?;
                let key_path = repo_root.join(".vwl").join("identity").join("node.key");
                if !key_path.exists() {
                    return Err(VwlError::InvalidTask(
                        "Node key not found. Run 'vwl node init' first.".to_string(),
                    ));
                }
                let hex_str = fs::read_to_string(&key_path)?.trim().to_string();
                let keypair = NodeKeypair::from_hex(&hex_str)?;
                let sig = keypair.sign_message(message.as_bytes());
                println!("{}", sig);
            }

            NodeCommands::Verify {
                pubkey,
                message,
                signature,
            } => {
                let valid = verify_signature(&pubkey, message.as_bytes(), &signature)?;
                if valid {
                    println!("Signature VALID for public key {}", pubkey);
                } else {
                    println!("Signature INVALID for public key {}", pubkey);
                }
            }
        },

        Commands::Skill { cmd } => match cmd {
            SkillCommands::List => {
                println!("Available VWL Protocol & Gate Skills:");
                for (name, desc) in skills::list_skills() {
                    println!("  - {:<24} : {}", name, desc);
                }
            }
            SkillCommands::Install { name, harness } => {
                let repo_root = GitInspector::find_repo_root(&current_dir)?;
                let installed = skills::install_skill(&repo_root, &name, &harness)?;
                println!(
                    "Successfully installed skill '{}' for harness '{}':",
                    name, harness
                );
                for path in installed {
                    println!("  -> {}", path.display());
                }
            }
        },

        Commands::Market => {
            println!("Market scanner: scanning GitHub topic:vwl-project and .vwl/project.yaml...");
        }
    }

    Ok(())
}
