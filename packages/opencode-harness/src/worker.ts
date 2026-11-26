export interface TaskContext {
  taskId: string;
  objective: string;
  acceptance: string[];
  baseCommit: string;
  worktreePath: string;
}

export interface WorkerExecutionResult {
  taskId: string;
  candidateCommit: string;
  status: "success" | "failure";
  logs: string;
}

/**
 * Creates bounded context for a developer agent (WU-G0-010).
 * Strictly provides ONLY: Task, acceptance, canonical base, and repo/worktree path.
 * Omits conversational context and prior implementer chat.
 */
export function createBoundedWorkerContext(input: TaskContext): TaskContext {
  if (!input.taskId || input.taskId.trim() === "") {
    throw new Error("Task ID is required");
  }
  if (!input.baseCommit || input.baseCommit.trim() === "") {
    throw new Error("Base commit SHA is required");
  }
  if (!input.worktreePath || input.worktreePath.trim() === "") {
    throw new Error("Worktree path is required");
  }

  return {
    taskId: input.taskId.trim(),
    objective: input.objective.trim(),
    acceptance: [...input.acceptance],
    baseCommit: input.baseCommit.trim(),
    worktreePath: input.worktreePath.trim()
  };
}
