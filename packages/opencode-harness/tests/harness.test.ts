import { describe, it, expect } from "vitest";
import { createBoundedWorkerContext } from "../src/worker.js";
import { evaluateVerificationEvidence } from "../src/verifier.js";

describe("OpenCode Harness Tests (WU-G0-010 & WU-G0-011)", () => {
  it("creates bounded worker context with strictly required fields", () => {
    const ctx = createBoundedWorkerContext({
      taskId: "T-001",
      objective: "Implement CLI shell",
      acceptance: ["cargo test passes", "cargo clippy passes"],
      baseCommit: "abc1234",
      worktreePath: "/tmp/worktrees/T-001"
    });

    expect(ctx.taskId).toBe("T-001");
    expect(ctx.objective).toBe("Implement CLI shell");
    expect(ctx.acceptance.length).toBe(2);
    expect(ctx.baseCommit).toBe("abc1234");
    expect(ctx.worktreePath).toBe("/tmp/worktrees/T-001");
  });

  it("throws if taskId or baseCommit is missing", () => {
    expect(() =>
      createBoundedWorkerContext({
        taskId: "",
        objective: "test",
        acceptance: [],
        baseCommit: "abc",
        worktreePath: "/tmp"
      })
    ).toThrow("Task ID is required");

    expect(() =>
      createBoundedWorkerContext({
        taskId: "T-001",
        objective: "test",
        acceptance: [],
        baseCommit: "",
        worktreePath: "/tmp"
      })
    ).toThrow("Base commit SHA is required");
  });

  it("evaluates verification evidence independently into a Proof DTO", () => {
    const verdict = evaluateVerificationEvidence(
      {
        taskId: "T-001",
        baseCommit: "c0_hash",
        candidateCommit: "candidate_sha_123",
        acceptanceCriteria: ["all tests green"]
      },
      0,
      "test result: ok. 5 passed"
    );

    expect(verdict.subject).toBe("candidate_sha_123");
    expect(verdict.type).toBe("work");
    expect(verdict.verdict).toBe("pass");
    expect(verdict.evidence.command).toContain("cargo test");
  });

  it("evaluates failing exit code as fail verdict", () => {
    const verdict = evaluateVerificationEvidence(
      {
        taskId: "T-001",
        baseCommit: "c0_hash",
        candidateCommit: "candidate_sha_fail",
        acceptanceCriteria: ["all tests green"]
      },
      1,
      "FAILED tests"
    );

    expect(verdict.verdict).toBe("fail");
  });
});
