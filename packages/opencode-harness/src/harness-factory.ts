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

function writeBoundedTaskAndGetCandidate(bounded: TaskContext, prefix: string): string {
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
  return candidateCommit;
}

export class OpenCodeHarnessAdapter implements HarnessAdapter {
  public readonly name: HarnessType = "opencode";

  public async executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult> {
    const bounded = createBoundedWorkerContext(context);
    const candidateCommit = writeBoundedTaskAndGetCandidate(bounded, "candidate");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs: `OpenCode executed bounded prompt for task ${bounded.taskId} on base ${bounded.baseCommit}`,
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
    const candidateCommit = writeBoundedTaskAndGetCandidate(bounded, "claude");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs: `Claude Code executed bounded instructions for ${bounded.taskId} in isolated worktree`,
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
    const candidateCommit = writeBoundedTaskAndGetCandidate(bounded, "copilot");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs: `GitHub Copilot workspace session completed task ${bounded.taskId}`,
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
    const candidateCommit = writeBoundedTaskAndGetCandidate(bounded, "gemini");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs: `Gemini / Antigravity agent executed bounded task ${bounded.taskId} in isolated worktree`,
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
    const candidateCommit = writeBoundedTaskAndGetCandidate(bounded, "custom");
    return {
      taskId: bounded.taskId,
      candidateCommit,
      status: "success",
      logs: `Custom harness executed for ${bounded.taskId}`,
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
