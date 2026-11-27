import { describe, it, expect } from "vitest";
import { mkdtempSync, rmSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { findVwlBinary, runCli, runNodeFallback } from "../bin/vwl.js";

describe("@horia-ai/vwl NPX CLI Runner", () => {
  it("locates the compiled vwl binary in workspace target/debug", () => {
    const binary = findVwlBinary();
    expect(binary).not.toBeNull();
    expect(binary).toContain("vwl");
  });

  it("executes vwl --help successfully returning status code 0", () => {
    const status = runCli(["--help"]);
    expect(status).toBe(0);
  });

  it("executes vwl bootstrap --help successfully returning status code 0", () => {
    const status = runCli(["bootstrap", "--help"]);
    expect(status).toBe(0);
  });

  it("executes runNodeFallback with help and status cleanly", () => {
    expect(runNodeFallback(["--help"])).toBe(0);
    expect(runNodeFallback(["status"])).toBe(0);
  });

  it("executes runNodeFallback postinstall safely skipping own repo", () => {
    expect(runNodeFallback(["postinstall"])).toBe(0);
  });

  it("executes postinstall in downstream INIT_CWD to auto-bootstrap", () => {
    const tempDir = mkdtempSync(join(tmpdir(), "vwl-postinstall-test-"));
    const origInitCwd = process.env.INIT_CWD;
    try {
      process.env.INIT_CWD = tempDir;
      const status = runNodeFallback(["postinstall"]);
      expect(status).toBe(0);
      expect(existsSync(join(tempDir, ".vwl/gates.yaml"))).toBe(true);
      expect(existsSync(join(tempDir, ".vwl/project.yaml"))).toBe(true);
      expect(existsSync(join(tempDir, "AGENTS.md"))).toBe(true);
      expect(existsSync(join(tempDir, ".opencode/skills/repo-bootstrap-initializer/SKILL.md"))).toBe(true);
    } finally {
      process.env.INIT_CWD = origInitCwd;
      rmSync(tempDir, { recursive: true, force: true });
    }
  });
});
