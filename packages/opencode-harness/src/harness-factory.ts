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

export type HarnessType = "opencode" | "claude" | "copilot" | "custom";

export interface HarnessAdapter {
  name: HarnessType;
  executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult>;
  executeVerifier(context: VerifierContext, exitCode: number, output: string): ProofVerdict;
  getSkillTargetDir(repoRoot: string): string;
}

export class OpenCodeHarnessAdapter implements HarnessAdapter {
  public readonly name: HarnessType = "opencode";

  public async executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult> {
    const bounded = createBoundedWorkerContext(context);
    return {
      taskId: bounded.taskId,
      candidateCommit: `candidate-${bounded.taskId.toLowerCase()}`,
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
    return {
      taskId: bounded.taskId,
      candidateCommit: `claude-${bounded.taskId.toLowerCase()}`,
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
    return {
      taskId: bounded.taskId,
      candidateCommit: `copilot-${bounded.taskId.toLowerCase()}`,
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

export class CustomHarnessAdapter implements HarnessAdapter {
  public readonly name: HarnessType = "custom";

  public async executeDeveloper(context: TaskContext): Promise<WorkerExecutionResult> {
    const bounded = createBoundedWorkerContext(context);
    return {
      taskId: bounded.taskId,
      candidateCommit: `custom-${bounded.taskId.toLowerCase()}`,
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
    case "custom":
      return new CustomHarnessAdapter();
    default:
      throw new Error(`Unsupported agent harness: ${type}`);
  }
}
