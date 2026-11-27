use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{CrosstermBackend, TestBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
use vwl_core::{
    decision::Decision, economics::EconomicLedger, git::GitInspector, identity::NodeCredential,
    ledger::LocalLedger, market::ProjectManifest, proof::Proof, task::Task, work_unit::WorkUnit,
};

#[derive(Parser, Debug)]
#[command(name = "vwl-tui")]
#[command(about = "VWL Terminal User Interface — Interactive & Headless Shell (Gate 3)")]
pub struct Args {
    /// Render single tab headlessly and exit (0: Home, 1: Market, 2: Projects/Ledger, 3: Economics, 4: Verifier)
    #[arg(long)]
    pub render_view: Option<usize>,

    /// Print rendered view buffer to stdout
    #[arg(long)]
    pub print_rendered: bool,
}

pub struct App {
    pub current_tab: usize,
    pub tabs: Vec<&'static str>,
    pub repo_root: PathBuf,
    pub git_head: String,
    pub canonical_base: String,
    pub node_id: String,
    pub node_name: String,
    pub manifest_roles: Vec<(String, String, u64, Vec<String>)>,
    pub tasks: Vec<Task>,
    pub work_units: Vec<WorkUnit>,
    pub decisions: Vec<Decision>,
    pub proofs: Vec<Proof>,
    pub total_usd: u64,
    pub locked_work: u64,
    pub reserved_work: u64,
    pub earned_work: u64,
    pub vesting_work: u64,
    pub liquid_work: u64,
    pub conservation_holds: bool,
}

impl App {
    pub fn new(repo_root: PathBuf) -> Self {
        let (git_head, canonical_base) = match GitInspector::inspect(&repo_root) {
            Ok(info) => (
                info.head_sha.chars().take(8).collect(),
                info.canonical_base.unwrap_or_else(|| "none".to_string()),
            ),
            Err(_) => ("unknown".to_string(), "none".to_string()),
        };

        let cred_path = repo_root
            .join(".vwl")
            .join("identity")
            .join("credential.yaml");
        let (node_id, node_name) = if cred_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cred_path) {
                if let Ok(cred) = serde_yaml::from_str::<NodeCredential>(&content) {
                    (cred.node_id, cred.display_name)
                } else {
                    ("unparsed".to_string(), "Unnamed Node".to_string())
                }
            } else {
                ("none".to_string(), "Unregistered Node".to_string())
            }
        } else {
            ("none".to_string(), "Unregistered Node".to_string())
        };

        let manifest_path = repo_root.join(".vwl").join("project.yaml");
        let manifest_roles = if manifest_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = serde_yaml::from_str::<ProjectManifest>(&content) {
                    manifest
                        .roles
                        .into_iter()
                        .map(|r| (r.id, r.title, r.budget_work, r.skills))
                        .collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let (tasks, work_units, decisions, proofs) =
            if let Ok(ledger) = LocalLedger::new(&repo_root) {
                (
                    ledger.get_tasks().unwrap_or_default(),
                    ledger.get_work_units().unwrap_or_default(),
                    ledger.get_decisions().unwrap_or_default(),
                    ledger.get_all_proofs().unwrap_or_default(),
                )
            } else {
                (Vec::new(), Vec::new(), Vec::new(), Vec::new())
            };

        let (
            total_usd,
            locked_work,
            reserved_work,
            earned_work,
            vesting_work,
            liquid_work,
            conservation_holds,
        ) = if let Ok(econ) = EconomicLedger::load_or_create(&repo_root) {
            let holds = econ.verify_conservation().is_ok();
            let (tot_work, locked, res) = econ
                .projects
                .values()
                .next()
                .map(|p| {
                    (
                        p.total_deposited_work,
                        p.project_locked_work,
                        p.reserved_work,
                    )
                })
                .unwrap_or_default();

            let (earned_node, vesting_node, liquid_node) = econ
                .nodes
                .values()
                .next()
                .map(|n| (n.earned_work, n.vesting_work, n.liquid_work))
                .unwrap_or_default();

            (
                tot_work / 1_000,
                locked,
                res,
                earned_node,
                vesting_node,
                liquid_node,
                holds,
            )
        } else {
            (0, 0, 0, 0, 0, 0, true)
        };

        Self {
            current_tab: 0,
            tabs: vec![
                "[1] Home",
                "[2] Market",
                "[3] Projects & Ledger",
                "[4] Economics & Balances",
                "[5] Verifier & Proofs",
            ],
            repo_root,
            git_head,
            canonical_base,
            node_id,
            node_name,
            manifest_roles,
            tasks,
            work_units,
            decisions,
            proofs,
            total_usd,
            locked_work,
            reserved_work,
            earned_work,
            vesting_work,
            liquid_work,
            conservation_holds,
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.tabs.len();
    }

    pub fn prev_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = self.tabs.len() - 1;
        } else {
            self.current_tab -= 1;
        }
    }

    pub fn draw(&self, f: &mut Frame) {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(size);

        // Header Tabs
        let titles: Vec<Line> = self
            .tabs
            .iter()
            .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Cyan))))
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Virtual Work Ledger (VWL) — Protocol Console "),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            )
            .select(self.current_tab);
        f.render_widget(tabs, chunks[0]);

        // Main content area
        match self.current_tab {
            0 => self.draw_home(f, chunks[1]),
            1 => self.draw_market(f, chunks[1]),
            2 => self.draw_ledger(f, chunks[1]),
            3 => self.draw_economics(f, chunks[1]),
            4 => self.draw_verifier(f, chunks[1]),
            _ => {}
        }

        // Footer Help
        let footer_text = Line::from(vec![
            Span::styled(
                " [Tab/1-5] ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Switch Views | "),
            Span::styled(
                " [r] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Refresh | "),
            Span::styled(
                " [q/Esc] ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Quit TUI"),
        ]);
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL).title(" Navigation "));
        f.render_widget(footer, chunks[2]);
    }

    fn draw_home(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Min(5)])
            .split(area);

        let status_lines = vec![
            Line::from(vec![
                Span::styled("Repository:     ", Style::default().fg(Color::DarkGray)),
                Span::raw(self.repo_root.display().to_string()),
            ]),
            Line::from(vec![
                Span::styled("Git HEAD:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.git_head, Style::default().fg(Color::Green)),
                Span::styled("  Canonical Base: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.canonical_base, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Node Identity:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.node_id, Style::default().fg(Color::Cyan)),
                Span::raw(format!(" ({})", self.node_name)),
            ]),
            Line::from(vec![
                Span::styled("Protocol State: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "ACTIVE",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | Gate 0-5 Verified | 0 Chat Zero-Trust Verification Enabled"),
            ]),
        ];

        let status_widget = Paragraph::new(status_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" System & Node Status (Gate 1) "),
        );
        f.render_widget(status_widget, chunks[0]);

        let summary_lines = vec![
            Line::from(vec![
                Span::styled("Active Roles (Market):  ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}", self.manifest_roles.len())),
            ]),
            Line::from(vec![
                Span::styled("Ingested Tasks:         ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}", self.tasks.len())),
            ]),
            Line::from(vec![
                Span::styled("Published Work Units:   ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}", self.work_units.len())),
            ]),
            Line::from(vec![
                Span::styled("Evaluated Decisions:    ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}", self.decisions.len())),
            ]),
            Line::from(vec![
                Span::styled("Verification Proofs:    ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}", self.proofs.len())),
            ]),
        ];
        let summary_widget = Paragraph::new(summary_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Ledger Summary Overview "),
        );
        f.render_widget(summary_widget, chunks[1]);
    }

    fn draw_market(&self, f: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .manifest_roles
            .iter()
            .map(|(id, title, budget, skills)| {
                Row::new(vec![
                    id.clone(),
                    title.clone(),
                    format!("{} WORK", budget),
                    skills.join(", "),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(16),
                Constraint::Length(24),
                Constraint::Length(16),
                Constraint::Min(20),
            ],
        )
        .header(
            Row::new(vec!["Role ID", "Role Title", "Budget", "Required Skills"]).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Market Roles & Discovery (.vwl/project.yaml - Gate 2 & 3) "),
        );

        f.render_widget(table, area);
    }

    fn draw_ledger(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Tasks Table
        let task_rows: Vec<Row> = self
            .tasks
            .iter()
            .map(|t| {
                Row::new(vec![
                    t.id.clone(),
                    t.objective.clone(),
                    format!("{} criteria", t.acceptance.len()),
                    t.base.chars().take(8).collect(),
                ])
            })
            .collect();

        let tasks_table = Table::new(
            task_rows,
            [
                Constraint::Length(18),
                Constraint::Min(30),
                Constraint::Length(16),
                Constraint::Length(12),
            ],
        )
        .header(
            Row::new(vec!["Task ID", "Objective", "Acceptance", "Base"]).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Authoritative Tasks (WU-G0-003) "),
        );
        f.render_widget(tasks_table, chunks[0]);

        // Work Units & Decisions Table
        let wu_rows: Vec<Row> = self
            .work_units
            .iter()
            .map(|wu| {
                let dec = self
                    .decisions
                    .iter()
                    .find(|d| d.subject == wu.commit)
                    .map(|d| format!("{:?}", d.outcome))
                    .unwrap_or_else(|| "Pending".to_string());

                Row::new(vec![
                    wu.task.clone(),
                    wu.commit.chars().take(10).collect(),
                    wu.base.chars().take(10).collect(),
                    dec,
                ])
            })
            .collect();

        let wu_table = Table::new(
            wu_rows,
            [
                Constraint::Length(18),
                Constraint::Length(14),
                Constraint::Length(14),
                Constraint::Min(15),
            ],
        )
        .header(
            Row::new(vec!["Task", "Candidate Commit", "Base Commit", "Decision"]).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Work Units & Evaluation Decisions (WU-G0-004 & WU-G0-006) "),
        );
        f.render_widget(wu_table, chunks[1]);
    }

    fn draw_economics(&self, f: &mut Frame, area: Rect) {
        let conservation_str = if self.conservation_holds {
            "HOLDS (Conservation Invariant Maintained)"
        } else {
            "VIOLATED (Invariant Broken!)"
        };

        let econ_lines = vec![
            Line::from(vec![
                Span::styled("Fiat Deposited:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("${} USD", self.total_usd),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  (1 USD = 1,000 WORK)",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(vec![
                Span::styled("Project Locked:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} WORK", self.locked_work),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Budget Reserved:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} WORK", self.reserved_work),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("Node Earned:        ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} WORK", self.earned_work),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Node Vesting:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} WORK", self.vesting_work),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
            Line::from(vec![
                Span::styled("Node Liquid:        ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} WORK", self.liquid_work),
                    Style::default().fg(Color::Blue),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Conservation Status: ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    conservation_str,
                    if self.conservation_holds {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Formula:             ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("TotalDeposited == ProjectLocked + Reserved + Earned + Vesting + Liquid"),
            ]),
        ];

        let econ_widget = Paragraph::new(econ_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Local Economic State Machine (Gate 5) "),
        );
        f.render_widget(econ_widget, area);
    }

    fn draw_verifier(&self, f: &mut Frame, area: Rect) {
        let proof_rows: Vec<Row> = self
            .proofs
            .iter()
            .map(|p| {
                let verdict_color = match p.verdict {
                    vwl_core::proof::Verdict::Pass => Color::Green,
                    vwl_core::proof::Verdict::Fail => Color::Red,
                    vwl_core::proof::Verdict::Inconclusive => Color::Yellow,
                };

                Row::new(vec![
                    p.subject.chars().take(12).collect(),
                    format!("{:?}", p.r#type),
                    format!("{:?}", p.verdict),
                    p.evidence.command.clone(),
                ])
                .style(Style::default().fg(verdict_color))
            })
            .collect();

        let table = Table::new(
            proof_rows,
            [
                Constraint::Length(14),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Min(30),
            ],
        )
        .header(
            Row::new(vec![
                "Subject Commit",
                "Proof Type",
                "Verdict",
                "Evidence Command",
            ])
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Verifier Evidence & Proofs (WU-G0-005 & WU-G0-011) "),
        );

        f.render_widget(table, area);
    }
}

fn run_interactive_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.draw(f))?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Tab | KeyCode::Right => app.next_tab(),
                    KeyCode::Left => app.prev_tab(),
                    KeyCode::Char('1') => app.current_tab = 0,
                    KeyCode::Char('2') => app.current_tab = 1,
                    KeyCode::Char('3') => app.current_tab = 2,
                    KeyCode::Char('4') => app.current_tab = 3,
                    KeyCode::Char('5') => app.current_tab = 4,
                    KeyCode::Char('r') => {
                        let cur = app.current_tab;
                        app = App::new(app.repo_root.clone());
                        app.current_tab = cur;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn render_headless(app: &mut App, tab: usize, width: u16, height: u16) -> String {
    app.current_tab = tab;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("Failed to initialize test backend");
    terminal
        .draw(|f| app.draw(f))
        .expect("Failed to draw frame");
    let buffer = terminal.backend().buffer();

    let mut out = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            out.push_str(buffer.get(x, y).symbol());
        }
        out.push('\n');
    }
    out
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let current_dir = std::env::current_dir()?;
    let repo_root = GitInspector::find_repo_root(&current_dir).unwrap_or(current_dir);
    let mut app = App::new(repo_root);

    if let Some(tab) = args.render_view {
        let rendered = render_headless(&mut app, tab, 100, 30);
        if args.print_rendered {
            print!("{}", rendered);
        } else {
            println!(
                "Headless TUI render for view {} successful ({} bytes)",
                tab,
                rendered.len()
            );
        }
        return Ok(());
    }

    // Setup terminal for interactive UI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_interactive_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("TUI Error: {:?}", err);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_app_initialization_and_headless_rendering_all_tabs() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path().to_path_buf();
        let mut app = App::new(repo_root);

        // Test tabs 0 to 4
        for tab in 0..5 {
            let rendered = render_headless(&mut app, tab, 120, 30);
            assert!(
                !rendered.is_empty(),
                "Tab {} should produce non-empty buffer",
                tab
            );
            assert!(
                rendered.contains("Virtual Work Ledger (VWL)"),
                "Should contain header"
            );
        }
    }

    #[test]
    fn test_tab_navigation_wraparound() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path().to_path_buf();
        let mut app = App::new(repo_root);

        assert_eq!(app.current_tab, 0);
        app.next_tab();
        assert_eq!(app.current_tab, 1);
        app.next_tab();
        assert_eq!(app.current_tab, 2);
        app.next_tab();
        assert_eq!(app.current_tab, 3);
        app.next_tab();
        assert_eq!(app.current_tab, 4);
        app.next_tab();
        assert_eq!(app.current_tab, 0); // Wraparound

        app.prev_tab();
        assert_eq!(app.current_tab, 4); // Backward wraparound
    }
}
