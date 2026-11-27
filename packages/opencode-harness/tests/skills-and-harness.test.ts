import { describe, it, expect } from "vitest";
import { getHarnessAdapter } from "../src/harness-factory.js";
import { installSkill, listAvailableSkills } from "../src/skill-installer.js";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

describe("Agent Harness Factory & Skill Installer Tests", () => {
  it("provides adapters for opencode, claude, copilot, and custom", async () => {
    const opencode = getHarnessAdapter("opencode");
    expect(opencode.name).toBe("opencode");

    const claude = getHarnessAdapter("claude");
    expect(claude.name).toBe("claude");

    const copilot = getHarnessAdapter("copilot");
    expect(copilot.name).toBe("copilot");

    const custom = getHarnessAdapter("custom");
    expect(custom.name).toBe("custom");

    expect(() => getHarnessAdapter("unknown" as any)).toThrow("Unsupported agent harness");
  });

  it("lists available built-in skills", () => {
    const skills = listAvailableSkills();
    expect(skills).toContain("tdd-mutation-testing");
    expect(skills).toContain("bounded-verifier");
    expect(skills).toContain("economic-conservation");
    expect(skills).toContain("dex-liquidity-rules");
  });

  it("installs skill into target harness paths (Claude, Copilot, OpenCode)", () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "vwl-skill-test-"));

    const installed = installSkill(tmpDir, "tdd-mutation-testing", "all");
    expect(installed.length).toBe(3);

    // Verify Claude SKILL.md
    const claudeSkill = path.join(tmpDir, ".claude", "skills", "tdd-mutation-testing", "SKILL.md");
    expect(fs.existsSync(claudeSkill)).toBe(true);
    expect(fs.readFileSync(claudeSkill, "utf-8")).toContain("TDD & Mutation Testing Protocol");

    // Verify OpenCode SKILL.md
    const opencodeSkill = path.join(tmpDir, ".vwl", "skills", "tdd-mutation-testing", "SKILL.md");
    expect(fs.existsSync(opencodeSkill)).toBe(true);

    // Verify Copilot instructions
    const copilotInstructions = path.join(tmpDir, ".github", "copilot-instructions.md");
    expect(fs.existsSync(copilotInstructions)).toBe(true);
    expect(fs.readFileSync(copilotInstructions, "utf-8")).toContain("Skill: tdd-mutation-testing");

    // Cleanup
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  it("throws when installing unknown skill", () => {
    expect(() => installSkill("/tmp", "non-existent-skill")).toThrow("Unknown skill");
  });
});
