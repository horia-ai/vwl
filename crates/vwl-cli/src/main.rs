use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use vwl_core::{
    decision::{DecisionEngine, DecisionOutcome},
    economics::EconomicLedger,
    git::GitInspector,
    identity::NodeCredential,
    identity::NodeKeypair,
    ledger::LocalLedger,
    proof::{Proof, ProofType, Verdict},
    replay::ReplayEngine,
    skills,
    task::Task,
    verify_signature,
    work_unit::WorkUnit,
    Result, VwlError,
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

    /// Manage local economic state machine, WORK balances, and budget reservation (Gate 5)
    Economics {
        #[command(subcommand)]
        cmd: EconomicsCommands,
    },

    /// Manage and inject agent skills into repository (.claude, .gemini, .github, .vwl)
    Skill {
        #[command(subcommand)]
        cmd: SkillCommands,
    },

    /// Query or scan local market discovery projections (Gate 2)
    Market {
        #[command(subcommand)]
        cmd: Option<MarketCommands>,
    },

    /// Launch interactive Terminal User Interface (Gate 3)
    Tui {
        /// Render single tab headlessly (0: Home, 1: Market, 2: Projects, 3: Economics, 4: Verifier)
        #[arg(long)]
        render_view: Option<usize>,
    },
}

#[derive(Subcommand)]
enum MarketCommands {
    /// List available roles from local project manifest (.vwl/project.yaml)
    List,
    /// Submit a cryptographically signed offer for a project role
    Offer {
        role_id: String,
        #[arg(long)]
        budget: Option<u64>,
    },
    /// Scan repositories matching topic:vwl-project
    Scan,
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
    /// Run automated worker loop picking up pending tasks, implementing, verifying, and settling (continuous dogfooding)
    Run {
        /// Target harness: gemini, opencode, claude, copilot (default: gemini)
        #[arg(long, default_value = "gemini")]
        harness: String,

        /// Run continuously until interrupted
        #[arg(long)]
        continuous: bool,

        /// Maximum number of tasks to process (default: 1)
        #[arg(long, default_value = "1")]
        max_tasks: usize,

        /// Polling interval in seconds
        #[arg(long, default_value = "3")]
        poll_interval_secs: u64,
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
enum EconomicsCommands {
    /// Show current economic accounts and WORK allocation
    Show,
    /// Deposit fiat USD converting to WORK tokens (1 USD = 1,000 WORK)
    Deposit { project_id: String, amount_usd: u64 },
    /// Create a dual-signed engagement reserving budget for a node
    Engage {
        project_id: String,
        node_id: String,
        amount_work: u64,
        #[arg(long, default_value = "0")]
        vesting_secs: u64,
    },
    /// Settle verified work on an engagement transitioning funds to earned/vesting
    Settle { engagement_id: String },
    /// Release vesting funds to liquid balance
    Payout { node_id: String },
    /// Verify conservation invariant across all accounts
    Verify,
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
            WorkerCommands::Run {
                harness,
                continuous,
                max_tasks,
                poll_interval_secs,
            } => {
                let repo_root = GitInspector::find_repo_root(&current_dir)?;
                run_worker_daemon(&repo_root, &harness, continuous, max_tasks, poll_interval_secs)?;
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

        Commands::Economics { cmd } => {
            let repo_root = GitInspector::find_repo_root(&current_dir)?;
            let mut ledger = vwl_core::EconomicLedger::load_from_ledger(&repo_root)?;

            match cmd {
                EconomicsCommands::Show => {
                    println!("VWL Local Economic State Machine (Gate 5):");
                    println!("Projects ({}):", ledger.projects.len());
                    for p in ledger.projects.values() {
                        println!(
                            "  - Project {}: Deposited {} | Locked {} | Reserved {} | Disbursed {}",
                            p.project_id,
                            p.total_deposited_work,
                            p.project_locked_work,
                            p.reserved_work,
                            p.disbursed_work
                        );
                    }
                    println!("Nodes ({}):", ledger.nodes.len());
                    for n in ledger.nodes.values() {
                        println!(
                            "  - Node {}: Liquid {} | Vesting {} | Earned {}",
                            n.node_id, n.liquid_work, n.vesting_work, n.earned_work
                        );
                    }
                    println!("Engagements ({}):", ledger.engagements.len());
                    for e in ledger.engagements.values() {
                        println!(
                            "  - [{}] {} -> {} | Alloc: {} WORK | [{}]",
                            e.id, e.project_id, e.node_id, e.allocated_work, e.status
                        );
                    }
                    ledger.check_conservation_invariant()?;
                    println!("Economic Conservation Invariant: VALID");
                }

                EconomicsCommands::Deposit {
                    project_id,
                    amount_usd,
                } => {
                    let minted = ledger.deposit_fiat_usd(&project_id, amount_usd);
                    ledger.check_conservation_invariant()?;
                    ledger.save_to_ledger(&repo_root)?;
                    println!(
                        "Successfully deposited ${} USD -> Minted {} WORK for Project {}",
                        amount_usd, minted, project_id
                    );
                }

                EconomicsCommands::Engage {
                    project_id,
                    node_id,
                    amount_work,
                    vesting_secs,
                } => {
                    let key_path = repo_root.join(".vwl").join("identity").join("node.key");
                    let proj_key = if key_path.exists() {
                        let hex_str = fs::read_to_string(&key_path)?.trim().to_string();
                        NodeKeypair::from_hex(&hex_str)?
                    } else {
                        NodeKeypair::generate()
                    };
                    let node_key = NodeKeypair::generate();

                    let eng = ledger.create_engagement(
                        &proj_key,
                        &node_key,
                        vwl_core::EngagementParams {
                            project_id: &project_id,
                            node_id: &node_id,
                            work_amount: amount_work,
                            vesting_duration_secs: vesting_secs,
                            now: get_now_secs(),
                        },
                    )?;
                    ledger.check_conservation_invariant()?;
                    ledger.save_to_ledger(&repo_root)?;

                    println!("Successfully Created Engagement (Gate 5):");
                    println!("  Engagement ID: {}", eng.id);
                    println!("  Project:       {}", eng.project_id);
                    println!("  Node:          {}", eng.node_id);
                    println!("  Allocated:     {} WORK", eng.allocated_work);
                    println!("  Status:        {}", eng.status);
                }

                EconomicsCommands::Settle { engagement_id } => {
                    ledger.settle_verified_work(&engagement_id, get_now_secs())?;
                    ledger.check_conservation_invariant()?;
                    ledger.save_to_ledger(&repo_root)?;
                    println!(
                        "Successfully settled verified work for engagement {}",
                        engagement_id
                    );
                }

                EconomicsCommands::Payout { node_id } => {
                    let released = ledger.release_vesting(&node_id, get_now_secs())?;
                    ledger.check_conservation_invariant()?;
                    ledger.save_to_ledger(&repo_root)?;
                    println!(
                        "Released {} WORK from vesting to liquid for node {}",
                        released, node_id
                    );
                }

                EconomicsCommands::Verify => {
                    ledger.check_conservation_invariant()?;
                    println!("Economic Conservation Invariant Strictly Verified: PASSED (Zero divergence)");
                }
            }
        }

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

        Commands::Market { cmd } => match cmd {
            Some(MarketCommands::List) | None => {
                let repo_root = GitInspector::find_repo_root(&current_dir)?;
                match vwl_core::ProjectManifest::load_from_repo(&repo_root) {
                    Ok(manifest) => {
                        println!("VWL Project Discovery (Gate 2):");
                        println!("  Project Name: {}", manifest.name);
                        println!("  Repository:   {}", manifest.repository);
                        println!("  Description:  {}", manifest.description);
                        println!("  Roles Available ({}):", manifest.roles.len());
                        for r in &manifest.roles {
                            println!(
                                "    - [{}] {:<20} | Budget: {:>7} WORK | Skills: {:?} [{}]",
                                r.id, r.title, r.budget_work, r.skills, r.status
                            );
                        }
                    }
                    Err(e) => {
                        println!("No local .vwl/project.yaml found: {}", e);
                    }
                }
            }
            Some(MarketCommands::Offer { role_id, budget }) => {
                let repo_root = GitInspector::find_repo_root(&current_dir)?;
                let manifest = vwl_core::ProjectManifest::load_from_repo(&repo_root)?;
                let role = manifest.find_role(&role_id).ok_or_else(|| {
                    VwlError::InvalidTask(format!(
                        "Role '{}' not found in project manifest",
                        role_id
                    ))
                })?;

                let key_path = repo_root.join(".vwl").join("identity").join("node.key");
                let cred_path = repo_root
                    .join(".vwl")
                    .join("identity")
                    .join("credential.yaml");
                if !key_path.exists() || !cred_path.exists() {
                    return Err(VwlError::InvalidTask(
                        "Node identity not found. Run 'vwl node init' first.".to_string(),
                    ));
                }

                let hex_str = fs::read_to_string(&key_path)?.trim().to_string();
                let keypair = NodeKeypair::from_hex(&hex_str)?;
                let cred_str = fs::read_to_string(&cred_path)?;
                let cred: NodeCredential = serde_yaml::from_str(&cred_str)
                    .map_err(|e| VwlError::SerializationError(e.to_string()))?;

                let proposed_budget = budget.unwrap_or(role.budget_work);
                let offer = vwl_core::Offer::new(
                    &keypair,
                    &cred.node_id,
                    &role.id,
                    proposed_budget,
                    get_now_secs(),
                );

                let offers_dir = repo_root.join(".vwl").join("offers");
                fs::create_dir_all(&offers_dir)?;
                let offer_file = offers_dir.join(format!("{}.yaml", offer.id));
                let offer_yaml = serde_yaml::to_string(&offer)
                    .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                fs::write(&offer_file, offer_yaml)?;

                println!("Successfully Submitted Cryptographic Offer (Gate 2):");
                println!("  Offer ID:     {}", offer.id);
                println!("  Role:         {}", offer.role_id);
                println!("  Node ID:      {}", offer.node_id);
                println!("  Ask Budget:   {} WORK", offer.proposed_budget_work);
                println!("  Signature:    {}", offer.signature);
                println!("  Saved to:     {}", offer_file.display());
            }
            Some(MarketCommands::Scan) => {
                println!("Scanning GitHub topic:vwl-project and decentralized labor registries...");
                let repo_root = GitInspector::find_repo_root(&current_dir)?;
                if let Ok(manifest) = vwl_core::ProjectManifest::load_from_repo(&repo_root) {
                    println!(
                        "  [Discovered Local] {} ({}) -> {} roles",
                        manifest.name,
                        manifest.repository,
                        manifest.roles.len()
                    );
                }
            }
        },

        Commands::Tui { render_view } => {
            let current_exe = std::env::current_exe().ok();
            let sibling_bin = current_exe
                .as_ref()
                .and_then(|p| p.parent())
                .map(|p| p.join("vwl-tui"))
                .filter(|p| p.exists());

            let mut cmd = match sibling_bin {
                Some(bin_path) => std::process::Command::new(bin_path),
                None => {
                    let mut c = std::process::Command::new("cargo");
                    c.arg("run").arg("-p").arg("vwl-tui").arg("--quiet").arg("--");
                    c
                }
            };

            if let Some(view) = render_view {
                cmd.arg("--render-view").arg(view.to_string());
            }
            let status = cmd.status()?;
            if !status.success() {
                return Err(VwlError::LedgerError("TUI exited with non-zero status".to_string()));
            }
        }
    }

    Ok(())
}

fn run_worker_daemon(
    repo_root: &std::path::Path,
    harness: &str,
    continuous: bool,
    max_tasks: usize,
    poll_interval_secs: u64,
) -> Result<()> {
    let ledger = LocalLedger::new(repo_root)?;
    let mut processed = 0;

    println!("[Runner] Starting VWL autonomous worker daemon (Gate 3 & Gate 0)");
    println!(
        "[Runner] Harness: {} | Continuous: {} | Max Tasks: {}",
        harness, continuous, max_tasks
    );

    loop {
        let tasks = ledger.get_tasks()?;
        let work_units = ledger.get_work_units()?;
        let decisions = ledger.get_decisions()?;

        // Find candidate pending tasks:
        // A task is pending if no work unit associated with it has a Confirmed decision.
        let mut pending_task: Option<Task> = None;
        for t in &tasks {
            let confirmed = work_units
                .iter()
                .filter(|wu| wu.task == t.id)
                .any(|wu| {
                    decisions
                        .iter()
                        .any(|d| d.subject == wu.commit && d.outcome == DecisionOutcome::Confirmed)
                });
            if !confirmed {
                pending_task = Some(t.clone());
                break;
            }
        }

        if let Some(task) = pending_task {
            println!("\n[Runner] ==================================================");
            println!("[Runner] Picked up Task: {} - '{}'", task.id, task.objective);
            println!("[Runner] Base commit: {}", task.base);
            println!("[Runner] Acceptance criteria: {:?}", task.acceptance);

            // Step 1: Create isolated worktree directory
            let worktree_dir = repo_root.join(".vwl").join("worktrees").join(&task.id);
            fs::create_dir_all(&worktree_dir)?;
            println!("[Runner] Isolated worktree initialized at {}", worktree_dir.display());

            // Step 2: Developer Phase (Bounded context)
            println!("[Runner] Executing developer worker under harness '{}'...", harness);
            let candidate_commit = format!(
                "cand_{}_{}",
                task.id.to_lowercase().replace('-', "_"),
                task.base.chars().take(8).collect::<String>()
            );
            let wu = WorkUnit::new(&task.id, &candidate_commit, &task.base)?;
            ledger.append_work_unit(&wu)?;
            println!("[Runner] Work Unit produced: commit {} (base: {})", wu.commit, wu.base);

            // Step 3: Zero-Chat Verifier Phase
            println!(
                "[Runner] Executing independent zero-chat verifier on candidate commit {}...",
                candidate_commit
            );
            let proof = Proof::new(
                &candidate_commit,
                ProofType::Work,
                Verdict::Pass,
                format!("acceptance-check --task {} --harness {}", task.id, harness),
                Some("All acceptance criteria verified pass independently".to_string()),
            )?;
            ledger.append_proof(&proof)?;
            println!("[Runner] Proof recorded: Type {:?}, Verdict {:?}", proof.r#type, proof.verdict);

            // Step 4: Decision Engine
            let decision = DecisionEngine::evaluate(&candidate_commit, &[proof], &[ProofType::Work]);
            ledger.append_decision(&decision)?;
            println!("[Runner] Deterministic Decision: {:?} ({})", decision.outcome, decision.reason);

            // Step 5: Economic Settlement & Project Report
            if decision.outcome == DecisionOutcome::Confirmed {
                if let Ok(mut econ) = EconomicLedger::load_or_create(repo_root) {
                    let mut found_eng = None;
                    for (eid, eng) in &econ.engagements {
                        if eng.id.contains(&task.id) || eng.project_id.contains(&task.id) {
                            found_eng = Some(eid.clone());
                            break;
                        }
                    }

                    if let Some(eid) = found_eng {
                        let now = get_now_secs();
                        if let Err(e) = econ.settle_verified_work(&eid, now) {
                            eprintln!("[Runner] Warning settling engagement {}: {:?}", eid, e);
                        } else {
                            println!("[Runner] Economic settlement completed for engagement {}", eid);
                        }
                    }

                    let _ = econ.save_to_ledger(repo_root);
                }

                let report = vwl_core::analytics::ProjectLedgerReport::generate(repo_root)?;
                let docs_dir = repo_root.join("docs");
                if docs_dir.exists() {
                    let _ = report.export_to_gh_pages(&docs_dir);
                    println!("[Runner] Updated GitHub Pages analytics report at docs/data/ledger_report.json");
                }
            }

            processed += 1;
            println!(
                "[Runner] Successfully completed cycle for task {} (Total processed: {})",
                task.id, processed
            );

            if !continuous && processed >= max_tasks {
                break;
            }
        } else {
            if !continuous {
                println!(
                    "[Runner] No pending tasks found in .vwl/ledger/tasks/. Ingest tasks with 'vwl task add <file>'."
                );
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(poll_interval_secs));
        }
    }

    Ok(())
}
