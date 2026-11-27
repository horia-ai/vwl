use crate::error::{Result, VwlError};
use std::fs;
use std::path::{Path, PathBuf};

pub struct SkillDef {
    pub name: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

pub const SKILLS: &[SkillDef] = &[
    SkillDef {
        name: "tdd-mutation-testing",
        description: "Enforce strict Test-Driven Development and Stryker mutation testing (>=80%)",
        content: "# TDD & Mutation Testing Protocol\n1. RED: Failing test first.\n2. GREEN: Minimal production code.\n3. REFACTOR: Clean code & types.\n4. MUTATE: Stryker mutation score >= 80% with zero surviving mutants.",
    },
    SkillDef {
        name: "bounded-verifier",
        description: "Execute independent verification receiving zero implementer conversational context",
        content: "# Independent Verifier Protocol\n1. Receive only Task, acceptance, base commit, and candidate diff.\n2. Zero implementer chat delivered.\n3. Execute acceptance commands and output structured Proof DTO.",
    },
    SkillDef {
        name: "economic-conservation",
        description: "Enforce WORK token conservation invariants across all transitions (Gate 5)",
        content: "# Economic Conservation Rules\nTotal WORK = Liquid + ProjectLocked + Reserved + Earned + Vesting.\nNo unauthorized inflation or discretionary balance changes.",
    },
    SkillDef {
        name: "dex-liquidity-rules",
        description: "Uniswap v3 concentrated liquidity pairing and Gate 10 public liquidity checks",
        content: "# DEX Liquidity Rules\n1 USD = 1,000 WORK (0.001 USD per WORK).\nPublic liquidity requires formal verification and decoupled floating rates.",
    },
    SkillDef {
        name: "repo-bootstrap-initializer",
        description: "Autonomous agent skill to initialize VWL, inspect languages, generate and validate .vwl/gates.yaml, setup git pre-commit hooks, and commit baseline configuration",
        content: "# VWL Repository Bootstrap & Initializer Protocol\n\n## 1. Readiness Detection\nCheck whether the current repository has VWL initialized and verified:\n- If '.vwl/gates.yaml' is missing or baseline gates are unverified, run 'vwl bootstrap --harness <harness>'.\n- Verify '.git/hooks/pre-commit' is present and executable.\n- Verify node credential exists in '.vwl/identity/credential.yaml'.\n\n## 2. Deterministic Static Gate Validation\n- Inspect discovered gates in '.vwl/gates.yaml'. Ensure every precommit gate executes a static, predictable toolchain command.\n- Run 'vwl verify precommit' on the current codebase to validate that all manual gates pass on the clean base.\n- If any gate fails, adjust '.vwl/gates.yaml' or fix repository baseline defects before proceeding.\n\n## 3. Baseline Commit & Task Execution\n- Once all gates pass cleanly, stage and commit the bootstrap files ('git add .vwl .git/hooks/pre-commit && git commit -m \"chore(vwl): bootstrap VWL protocol gates and skills\"').\n- Ingest verified tasks with 'vwl task add <spec.yaml> --skeptic'.\n- Launch worker loop with 'vwl worker run --role developer'.",
    },
    SkillDef {
        name: "validator-wiring",
        description: "Specialist validator proving real runtime path reachability, route registration, and module linkage (Gate 14)",
        content: "# Validator Skill: Wiring Proof (Gate 14)\n\n## Purpose\nProves implementation is actively wired and reachable through the intended runtime path, not merely invoked in isolated synthetic unit tests.\n\n## Verification Rules\n1. Inspect candidate diff for real wiring points (route registration, dependency injection, CLI commands, event listeners).\n2. Execute end-to-end wiring tests or symbol link checks.\n3. Fail if behavior exists in isolation without active integration.\n4. Emit structured Proof with gate='wiring' and plane=Deterministic.",
    },
    SkillDef {
        name: "validator-test-integrity",
        description: "Specialist validator detecting test/oracle relaxation, weakened assertions, and mock abuse (Gate 15)",
        content: "# Validator Skill: Test Integrity Gate (Gate 15)\n\n## Purpose\nSurfaces and prevents test or oracle relaxation where tests are weakened simply to let an improper candidate pass.\n\n## Detection Invariants\n1. Check for deleted or commented-out assertion statements.\n2. Flag inflated timeouts, loosened property ranges, or catch-all exception swallows.\n3. Verify test assertions check real invariants rather than echo implementation output.\n4. Emit structured Proof or Challenge with reproducer artifact if tests are degraded.",
    },
    SkillDef {
        name: "validator-targeted-mutation",
        description: "Specialist validator executing targeted mutation analysis against modified lines and evaluating survivors (Gate 16)",
        content: "# Validator Skill: Targeted Mutation (Gate 16)\n\n## Purpose\nValidates test rigor against candidate diffs using automated fault injection (Stryker / cargo-mutants).\n\n## Protocol\n1. Target mutation strictly to modified lines, branches, and affected test suites.\n2. Require minimum mutation score threshold (>=80%).\n3. Analyze surviving mutants: distinguish genuine behavioral escapes from equivalent mutants.\n4. Emit structured Proof with gate='mutation-targeted' and survivor analysis.",
    },
    SkillDef {
        name: "validator-adversarial-skeptic",
        description: "Specialist validator performing adversarial judgment evaluation for edge cases and spec drift (Plane 1)",
        content: "# Validator Skill: Adversarial Skeptic (Plane 1 Judgment)\n\n## Purpose\nAdversarial evaluation testing boundary conditions, unstated assumptions, and conversational leakage.\n\n## Checklist\n1. Task self-containment: Reject dependencies on implementer chat or session memory.\n2. Boundary probing: Test zero/nil/empty, overflow, concurrency, and timeout conditions.\n3. Candidate isolation: Never rewrite candidate code; report defects through structured Challenge.\n4. Emit structured Proof with plane=Judgment.",
    },
    SkillDef {
        name: "validator-shadow-smoke",
        description: "Specialist validator executing read-only shadow traffic and isolated deployment smoke verification (Gate 18 & 19)",
        content: "# Validator Skill: Shadow Traffic & Smoke Contract (Gate 18 & 19)\n\n## Purpose\nValidate candidate in containerized smoke/deployment and mirrored traffic without production state mutation.\n\n## Safety Invariants\n1. Read-only execution: Mutation calls MUST be rejected or mocked.\n2. Side-effect suppression: External webhooks, emails, and database writes suppressed.\n3. Bounded execution: Enforce request limit and duration timeouts.\n4. Emit structured Proof with gate='deployment-shadow'.",
    },
];

pub fn list_skills() -> Vec<(&'static str, &'static str)> {
    SKILLS.iter().map(|s| (s.name, s.description)).collect()
}

pub fn install_skill(repo_root: &Path, skill_name: &str, harness: &str) -> Result<Vec<PathBuf>> {
    let skill = SKILLS
        .iter()
        .find(|s| s.name == skill_name)
        .ok_or_else(|| VwlError::InvalidTask(format!("Unknown skill '{}'", skill_name)))?;

    let mut installed = Vec::new();
    let harnesses: Vec<&str> = if harness == "all" {
        vec!["opencode", "claude", "copilot", "gemini"]
    } else {
        vec![harness]
    };

    for h in harnesses {
        match h {
            "opencode" => {
                let doc = format!(
                    "---\nname: {}\ndescription: {}\n---\n\n{}\n",
                    skill.name, skill.description, skill.content
                );
                // Install into .vwl/skills/<name>/SKILL.md
                let vwl_dir = repo_root.join(".vwl").join("skills").join(skill.name);
                fs::create_dir_all(&vwl_dir)?;
                let file = vwl_dir.join("SKILL.md");
                fs::write(&file, &doc)?;
                installed.push(file);

                // Also install into .opencode/skills/<name>/SKILL.md for OpenCode auto-discovery
                let opencode_dir = repo_root.join(".opencode").join("skills").join(skill.name);
                fs::create_dir_all(&opencode_dir)?;
                let oc_file = opencode_dir.join("SKILL.md");
                fs::write(&oc_file, &doc)?;
                installed.push(oc_file);
            }
            "claude" => {
                let target_dir = repo_root.join(".claude").join("skills").join(skill.name);
                fs::create_dir_all(&target_dir)?;
                let file = target_dir.join("SKILL.md");
                let doc = format!(
                    "---\nname: {}\ndescription: {}\n---\n\n{}\n",
                    skill.name, skill.description, skill.content
                );
                fs::write(&file, doc)?;
                installed.push(file);
            }
            "gemini" => {
                let target_dir = repo_root.join(".gemini").join("skills").join(skill.name);
                fs::create_dir_all(&target_dir)?;
                let file = target_dir.join("SKILL.md");
                let doc = format!(
                    "---\nname: {}\ndescription: {}\n---\n\n{}\n",
                    skill.name, skill.description, skill.content
                );
                fs::write(&file, doc)?;
                installed.push(file);
            }
            "copilot" => {
                let target_dir = repo_root.join(".github");
                fs::create_dir_all(&target_dir)?;
                let file = target_dir.join("copilot-instructions.md");
                let block = format!("\n\n## Skill: {}\n{}\n", skill.name, skill.content);
                let mut current = if file.exists() {
                    fs::read_to_string(&file)?
                } else {
                    String::new()
                };
                current.push_str(&block);
                fs::write(&file, current)?;
                installed.push(file);
            }
            _ => {
                let target_dir = repo_root
                    .join(".vwl")
                    .join("custom-skills")
                    .join(skill.name);
                fs::create_dir_all(&target_dir)?;
                let file = target_dir.join("SKILL.md");
                fs::write(&file, skill.content)?;
                installed.push(file);
            }
        }
    }

    Ok(installed)
}

pub fn install_all_skills(repo_root: &Path, harness: &str) -> Result<Vec<PathBuf>> {
    let mut all_installed = Vec::new();
    for skill in SKILLS {
        let mut paths = install_skill(repo_root, skill.name, harness)?;
        all_installed.append(&mut paths);
    }
    Ok(all_installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_list_skills() {
        let skills = list_skills();
        assert_eq!(skills.len(), 10);
        assert!(skills
            .iter()
            .any(|(name, _)| *name == "tdd-mutation-testing"));
        assert!(skills
            .iter()
            .any(|(name, _)| *name == "repo-bootstrap-initializer"));
        assert!(skills.iter().any(|(name, _)| *name == "bounded-verifier"));
        assert!(skills.iter().any(|(name, _)| *name == "validator-wiring"));
        assert!(skills
            .iter()
            .any(|(name, _)| *name == "validator-test-integrity"));
        assert!(skills
            .iter()
            .any(|(name, _)| *name == "validator-targeted-mutation"));
        assert!(skills
            .iter()
            .any(|(name, _)| *name == "validator-adversarial-skeptic"));
        assert!(skills
            .iter()
            .any(|(name, _)| *name == "validator-shadow-smoke"));
    }

    #[test]
    fn test_install_unknown_skill() {
        let temp = tempdir().unwrap();
        let err = install_skill(temp.path(), "non-existent-skill", "all");
        assert!(err.is_err());
    }

    #[test]
    fn test_install_opencode_skill() {
        let temp = tempdir().unwrap();
        let paths = install_skill(temp.path(), "tdd-mutation-testing", "opencode").unwrap();
        assert_eq!(paths.len(), 2);
        let content = fs::read_to_string(&paths[0]).unwrap();
        assert!(content.contains("name: tdd-mutation-testing"));
        assert!(content.contains("Stryker mutation score"));
        let oc_content = fs::read_to_string(&paths[1]).unwrap();
        assert!(oc_content.contains("name: tdd-mutation-testing"));
    }

    #[test]
    fn test_install_claude_skill() {
        let temp = tempdir().unwrap();
        let paths = install_skill(temp.path(), "bounded-verifier", "claude").unwrap();
        assert_eq!(paths.len(), 1);
        let content = fs::read_to_string(&paths[0]).unwrap();
        assert!(content.contains("name: bounded-verifier"));
        assert!(content.contains("Zero implementer chat"));
    }

    #[test]
    fn test_install_gemini_skill() {
        let temp = tempdir().unwrap();
        let paths = install_skill(temp.path(), "economic-conservation", "gemini").unwrap();
        assert_eq!(paths.len(), 1);
        let content = fs::read_to_string(&paths[0]).unwrap();
        assert!(content.contains("name: economic-conservation"));
        assert!(content.contains("Total WORK = Liquid"));
    }

    #[test]
    fn test_install_copilot_skill() {
        let temp = tempdir().unwrap();
        let paths = install_skill(temp.path(), "economic-conservation", "copilot").unwrap();
        assert_eq!(paths.len(), 1);
        let content = fs::read_to_string(&paths[0]).unwrap();
        assert!(content.contains("## Skill: economic-conservation"));
        assert!(content.contains("Total WORK = Liquid"));
    }

    #[test]
    fn test_install_all_and_custom_skill() {
        let temp = tempdir().unwrap();
        let paths = install_skill(temp.path(), "dex-liquidity-rules", "all").unwrap();
        assert_eq!(paths.len(), 5);

        let custom_paths = install_skill(temp.path(), "dex-liquidity-rules", "my-custom").unwrap();
        assert_eq!(custom_paths.len(), 1);
        assert!(custom_paths[0].ends_with(".vwl/custom-skills/dex-liquidity-rules/SKILL.md"));

        let all_skills = install_all_skills(temp.path(), "gemini").unwrap();
        assert_eq!(all_skills.len(), 10);
    }
}
