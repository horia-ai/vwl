import fs from "node:fs";
import path from "node:path";
import { HarnessType, getHarnessAdapter } from "./harness-factory.js";

export interface SkillDefinition {
  name: string;
  description: string;
  instructions: string;
}

export const BUILTIN_SKILLS: Record<string, SkillDefinition> = {
  "tdd-mutation-testing": {
    name: "tdd-mutation-testing",
    description: "Strict Test-Driven Development (TDD) and Stryker mutation testing (>= 80% threshold)",
    instructions: `# TDD & Mutation Testing Protocol
1. RED: Write unit and integration tests first. Assert failure.
2. GREEN: Minimal implementation code to make tests pass.
3. REFACTOR: Clean code, zero warnings (cargo clippy, eslint, typecheck).
4. MUTATE: Run Stryker mutation testing (npm run test:mutate). Score must be >= 80% with zero surviving boundary mutants.`,
  },
  "bounded-verifier": {
    name: "bounded-verifier",
    description: "Independent verifier operating with zero implementer chat context (WU-G0-011)",
    instructions: `# Independent Verifier Protocol
1. Context Isolation: Verifier receives only Task, acceptance criteria, base commit, candidate diff.
2. No Conversation: Zero conversational chat from developer implementer is provided.
3. Deterministic Proof: Run exact acceptance commands and output structured Proof DTO.`,
  },
  "economic-conservation": {
    name: "economic-conservation",
    description: "Enforces WORK token state conservation invariants across accounting states (Gate 5)",
    instructions: `# Economic Conservation Rules
Total Funded = Liquid + ProjectLocked + Reserved + Earned + Vesting
Every state transition must preserve total token balance. Nonce idempotency prevents double spending.`,
  },
  "dex-liquidity-rules": {
    name: "dex-liquidity-rules",
    description: "Uniswap v3 concentrated liquidity rules and Gate 10 public market checks",
    instructions: `# DEX Liquidity Invariants
1 USD = 1,000 WORK (1 WORK = 0.001 USD).
Public liquidity pools require decoupled floating pricing and formal security verification before deployment.`,
  },
  "spec-writer-adversarial-skeptic": {
    name: "spec-writer-adversarial-skeptic",
    description: "Adversarial spec writer that grills user with rigorous questions to extract falsifiable task specs",
    instructions: `# Adversarial Spec Writer & Skeptic Grilling Protocol
1. Grill user with pointed questions on falsifiability, invariants, edge cases, error codes, and canonical base commit.
2. Reject subjective phrases ('clean code', 'looks good') and conversational dependencies ('as discussed').
3. Formulate immutable YAML task spec and ingest via 'vwl task add <spec.yaml> --skeptic'.`,
  },
};

export function listAvailableSkills(): string[] {
  return Object.keys(BUILTIN_SKILLS);
}

export function installSkill(
  repoRoot: string,
  skillName: string,
  targetHarness: HarnessType | "all" = "all"
): string[] {
  const skill = BUILTIN_SKILLS[skillName];
  if (!skill) {
    throw new Error(`Unknown skill: '${skillName}'. Available: ${listAvailableSkills().join(", ")}`);
  }

  const harnesses: HarnessType[] =
    targetHarness === "all" ? ["opencode", "claude", "copilot", "gemini"] : [targetHarness];

  const installedPaths: string[] = [];

  for (const h of harnesses) {
    const adapter = getHarnessAdapter(h);
    const targetDir = adapter.getSkillTargetDir(repoRoot);

    if (h === "copilot") {
      // In Copilot, append or write to .github/copilot-instructions.md
      const githubDir = path.join(repoRoot, ".github");
      if (!fs.existsSync(githubDir)) {
        fs.mkdirSync(githubDir, { recursive: true });
      }
      const instructionsFile = path.join(githubDir, "copilot-instructions.md");
      const block = `\n\n## Skill: ${skill.name}\n${skill.instructions}\n`;
      fs.appendFileSync(instructionsFile, block, "utf-8");
      installedPaths.push(instructionsFile);
    } else {
      const skillFolder = path.join(targetDir, skill.name);
      if (!fs.existsSync(skillFolder)) {
        fs.mkdirSync(skillFolder, { recursive: true });
      }
      const skillFile = path.join(skillFolder, "SKILL.md");
      const content = `---
name: ${skill.name}
description: "${skill.description}"
---

${skill.instructions}
`;
      fs.writeFileSync(skillFile, content, "utf-8");
      installedPaths.push(skillFile);
    }
  }

  return installedPaths;
}
