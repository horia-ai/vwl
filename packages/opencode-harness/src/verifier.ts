export interface VerifierContext {
  taskId: string;
  baseCommit: string;
  candidateCommit: string;
  acceptanceCriteria: string[];
}

export interface ProofVerdict {
  subject: string;
  type: "work" | "impact" | "structural";
  verdict: "pass" | "fail" | "inconclusive";
  evidence: {
    command: string;
    output?: string;
  };
}

/**
 * Independent verifier wrapper (WU-G0-011).
 * Verifier receives only Task, acceptance criteria, base commit, and candidate commit/diff.
 * Zero implementer conversation is passed.
 */
export function evaluateVerificationEvidence(
  context: VerifierContext,
  exitCode: number,
  output: string
): ProofVerdict {
  if (!context.candidateCommit) {
    throw new Error("Candidate commit subject is required");
  }

  const verdict = exitCode === 0 ? "pass" : "fail";
  return {
    subject: context.candidateCommit,
    type: "work",
    verdict,
    evidence: {
      command: "cargo test && cargo clippy",
      output: output.slice(0, 1000)
    }
  };
}
