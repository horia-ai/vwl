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
                let target_dir = repo_root.join(".vwl").join("skills").join(skill.name);
                fs::create_dir_all(&target_dir)?;
                let file = target_dir.join("SKILL.md");
                let doc = format!(
                    "---\nname: {}\ndescription: {}\n---\n\n{}\n",
                    skill.name, skill.description, skill.content
                );
                fs::write(&file, doc)?;
                installed.push(file);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_list_skills() {
        let skills = list_skills();
        assert_eq!(skills.len(), 4);
        assert!(skills
            .iter()
            .any(|(name, _)| *name == "tdd-mutation-testing"));
        assert!(skills.iter().any(|(name, _)| *name == "bounded-verifier"));
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
        assert_eq!(paths.len(), 1);
        let content = fs::read_to_string(&paths[0]).unwrap();
        assert!(content.contains("name: tdd-mutation-testing"));
        assert!(content.contains("Stryker mutation score"));
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
        assert_eq!(paths.len(), 4);

        let custom_paths = install_skill(temp.path(), "dex-liquidity-rules", "my-custom").unwrap();
        assert_eq!(custom_paths.len(), 1);
        assert!(custom_paths[0].ends_with(".vwl/custom-skills/dex-liquidity-rules/SKILL.md"));
    }
}
