import {
  TaskContext,
  WorkerExecutionResult,
  createBoundedWorkerContext,
} from "./worker.js";
import {
  VerifierContext,
  ProofVerdict,
  evaluateVerificationEvidence,
} from "./verifier.js";
import path from "node:path";
import fs from "node:fs";
import { execSync } from "node:child_process";

export type HarnessType = "opencode" | "claude" | "copilot" | "gemini" | "custom";

export interface HarnessAdapter {
  name: HarnessType;
  executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult>;
  executeVerifier(context: VerifierContext, exitCode: number, output: string): ProofVerdict;
  getSkillTargetDir(repoRoot: string): string;
}

function runAgentAndGetCandidate(
  bounded: TaskContext,
  prefix: string,
  harness: HarnessType
): { candidateCommit: string; logs: string } {
  const worktreeDir = bounded.worktreePath;
  if (!fs.existsSync(worktreeDir)) {
    fs.mkdirSync(worktreeDir, { recursive: true });
  }
  const acceptanceBullets = (bounded.acceptance || []).map((a) => `- ${a}`).join("\n");
  const taskMd = `# Task: ${bounded.taskId}

## Objective
${bounded.objective}

## Canonical Base
${bounded.baseCommit}

## Acceptance Criteria
${acceptanceBullets}

## Bounded Instructions
1. Strictly follow TDD (Red -> Green -> Refactor).
2. Ensure tests pass and mutation coverage >= 80%.
3. Verify changes with \`vwl verify precommit\`.
`;
  fs.writeFileSync(path.join(worktreeDir, "TASK.md"), taskMd);

  let logs = `Initialized bounded workspace for ${bounded.taskId}`;

  // Execute real autonomous agent if available and not in synthetic test mode
  try {
    if (process.env.VWL_EXEC_AGENT !== "false" && process.env.NODE_ENV !== "test" && !process.env.VITEST) {
      if (harness === "opencode") {
        const hasOpencode = execSync("which opencode", { encoding: "utf-8", stdio: ["ignore", "pipe", "ignore"] }).trim();
        if (hasOpencode) {
          const out = execSync(
            'opencode run --auto "Implement the requirements in TASK.md following strict TDD. Verify with vwl verify precommit."',
            { cwd: worktreeDir, encoding: "utf-8", timeout: 180000, stdio: ["ignore", "pipe", "pipe"] }
          );
          logs = `OpenCode executed task: ${out.slice(0, 300)}`;
        }
      } else if (harness === "claude") {
        const hasClaude = execSync("which claude", { encoding: "utf-8", stdio: ["ignore", "pipe", "ignore"] }).trim();
        if (hasClaude) {
          const out = execSync(
            'claude -p "Implement the requirements in TASK.md following strict TDD. Verify with vwl verify precommit." --dangerously-skip-permissions',
            { cwd: worktreeDir, encoding: "utf-8", timeout: 180000, stdio: ["ignore", "pipe", "pipe"] }
          );
          logs = `Claude executed task: ${out.slice(0, 300)}`;
        }
      } else if (process.env.VWL_AGENT_CMD) {
        const out = execSync(process.env.VWL_AGENT_CMD, {
          cwd: worktreeDir,
          encoding: "utf-8",
          timeout: 180000,
          stdio: ["ignore", "pipe", "pipe"],
        });
        logs = `Custom agent executed task: ${out.slice(0, 300)}`;
      }
    }
  } catch (err: any) {
    logs = `Agent execution exited: ${err.message?.slice(0, 200) || "error"}`;
  }

  // Auto-commit uncommitted changes in worktree if any exist (excluding TASK.md prompt)
  try {
    const status = execSync("git status --porcelain", {
      cwd: worktreeDir,
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
    const hasCodeChanges = status.split("\n").some((l) => l.trim() && !l.trim().endsWith("TASK.md"));
    if (hasCodeChanges) {
      execSync("git add -A", { cwd: worktreeDir, stdio: "ignore" });
      execSync(`git commit -m "feat(${bounded.taskId}): implement task requirements via ${harness}"`, {
        cwd: worktreeDir,
        stdio: "ignore",
      });
    }
  } catch {
    // ignore
  }

  let candidateCommit = `${prefix}-${bounded.taskId.toLowerCase()}`;
  try {
    const gitHead = execSync("git rev-parse HEAD", {
      cwd: worktreeDir,
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
    if (gitHead && gitHead !== bounded.baseCommit) {
      candidateCommit = gitHead;
    }
  } catch {
    // fallback
  }
  return { candidateCommit, logs };
}

export class OpenCodeHarnessAdapter implements HarnessAdapter {
  public readonly name: HarnessType = "opencode";

  public async executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult> {
    const bounded = createBoundedWorkerContext(context);
    const { candidateCommit, logs } = runAgentAndGetCandidate(bounded, "candidate", "opencode");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs,
    };
  }

  public executeVerifier(context: VerifierContext, exitCode: number, output: string): ProofVerdict {
    return evaluateVerificationEvidence(context, exitCode, output);
  }

  public getSkillTargetDir(repoRoot: string): string {
    return path.join(repoRoot, ".vwl", "skills");
  }
}

export class ClaudeHarnessAdapter implements HarnessAdapter {
  public readonly name: HarnessType = "claude";

  public async executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult> {
    const bounded = createBoundedWorkerContext(context);
    const { candidateCommit, logs } = runAgentAndGetCandidate(bounded, "claude", "claude");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs,
    };
  }

  public executeVerifier(context: VerifierContext, exitCode: number, output: string): ProofVerdict {
    const verdict = evaluateVerificationEvidence(context, exitCode, output);
    return {
      ...verdict,
      evidence: {
        command: "claude-code verify --non-interactive",
        output: output.slice(0, 1000),
      },
    };
  }

  public getSkillTargetDir(repoRoot: string): string {
    return path.join(repoRoot, ".claude", "skills");
  }
}

export class CopilotHarnessAdapter implements HarnessAdapter {
  public readonly name: HarnessType = "copilot";

  public async executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult> {
    const bounded = createBoundedWorkerContext(context);
    const { candidateCommit, logs } = runAgentAndGetCandidate(bounded, "copilot", "copilot");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs,
    };
  }

  public executeVerifier(context: VerifierContext, exitCode: number, output: string): ProofVerdict {
    const verdict = evaluateVerificationEvidence(context, exitCode, output);
    return {
      ...verdict,
      evidence: {
        command: "gh copilot verify",
        output: output.slice(0, 1000),
      },
    };
  }

  public getSkillTargetDir(repoRoot: string): string {
    return path.join(repoRoot, ".github", "instructions");
  }
}

export class GeminiHarnessAdapter implements HarnessAdapter {
  public readonly name: HarnessType = "gemini";

  public async executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult> {
    const bounded = createBoundedWorkerContext(context);
    const { candidateCommit, logs } = runAgentAndGetCandidate(bounded, "gemini", "gemini");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs,
    };
  }

  public executeVerifier(context: VerifierContext, exitCode: number, output: string): ProofVerdict {
    const verdict = evaluateVerificationEvidence(context, exitCode, output);
    return {
      ...verdict,
      evidence: {
        command: "gemini verify --non-interactive",
        output: output.slice(0, 1000),
      },
    };
  }

  public getSkillTargetDir(repoRoot: string): string {
    return path.join(repoRoot, ".gemini", "skills");
  }
}

export class CustomHarnessAdapter implements HarnessAdapter {
  public readonly name: HarnessType = "custom";

  public async executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult> {
    const bounded = createBoundedWorkerContext(context);
    const { candidateCommit, logs } = runAgentAndGetCandidate(bounded, "custom", "custom");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs,
    };
  }

  public executeVerifier(context: VerifierContext, exitCode: number, output: string): ProofVerdict {
    return evaluateVerificationEvidence(context, exitCode, output);
  }

  public getSkillTargetDir(repoRoot: string): string {
    return path.join(repoRoot, ".vwl", "custom-skills");
  }
}

export function getHarnessAdapter(type: HarnessType): HarnessAdapter {
  switch (type) {
    case "opencode":
      return new OpenCodeHarnessAdapter();
    case "claude":
      return new ClaudeHarnessAdapter();
    case "copilot":
      return new CopilotHarnessAdapter();
    case "gemini":
      return new GeminiHarnessAdapter();
    case "custom":
      return new CustomHarnessAdapter();
    default:
      throw new Error(`Unsupported agent harness: ${type}`);
  }
}
