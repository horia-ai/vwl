use clap::Parser;
use vwl_tui::{run_tui_command, Args};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let current_dir = std::env::current_dir()?;
    let repo_root =
        vwl_core::git::GitInspector::find_repo_root(&current_dir).unwrap_or(current_dir);
    run_tui_command(repo_root, args.render_view)?;
    Ok(())
}
