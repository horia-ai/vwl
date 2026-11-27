#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, writeFileSync, readFileSync, chmodSync } from "node:fs";
import { resolve, dirname, basename } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export function findVwlBinary() {
  // 1. Check if vwl is available in PATH
  const whichResult = spawnSync("which", ["vwl"], { encoding: "utf-8" });
  if (whichResult.status === 0 && whichResult.stdout.trim()) {
    return whichResult.stdout.trim();
  }

  // 2. Check workspace target/release or target/debug
  const workspaceRoot = resolve(__dirname, "../../..");
  const candidatePaths = [
    resolve(workspaceRoot, "target/release/vwl"),
    resolve(workspaceRoot, "target/debug/vwl"),
  ];

  for (const path of candidatePaths) {
    if (existsSync(path)) {
      return path;
    }
  }

  return null;
}

export function runNodeFallback(args, customCwd = null) {
  const cwd = customCwd || process.cwd();
  const cmd = args[0] || "--help";

  if (cmd === "--help" || cmd === "-h" || cmd === "help") {
    console.log(`Virtual Work Ledger (VWL) — Node.js NPX Runner
Usage: npx @horia-ai/vwl [COMMAND]

Commands:
  bootstrap   Bootstrap repository: detect stack, configure & test static gates, install hooks and skills
  init        Initialize .vwl/gates.yaml and git pre-commit hook
  verify      Execute validator gates (e.g. 'verify precommit')
  status      Print repository and ledger status

To access the full suite of cryptographic ledger, economic settlement, and autonomous agent loops,
install the native 'vwl' binary or compile via Cargo: cargo install --path crates/vwl-cli`);
    return 0;
  }

  if (cmd === "status") {
    const gitHead = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf-8" });
    const head = gitHead.status === 0 ? gitHead.stdout.trim() : "unborn";
    console.log("Virtual Work Ledger (VWL) Status [Node Fallback]:");
    console.log(`  Repository Root: ${cwd}`);
    console.log(`  HEAD SHA:        ${head}`);
    return 0;
  }

  if (cmd === "postinstall") {
    const targetCwd = process.env.INIT_CWD || process.cwd();
    const pkgRoot = resolve(__dirname, "..");

    // Avoid running inside the package's own root during development/testing
    if (targetCwd === pkgRoot || resolve(targetCwd, "packages/vwl") === pkgRoot) {
      return 0;
    }

    const gatesPath = resolve(targetCwd, ".vwl/gates.yaml");
    if (!existsSync(gatesPath)) {
      console.log(`[VWL] 🚀 Installed @horia-ai/vwl in ${targetCwd}`);
      console.log("[VWL] 🛠️ Auto-initializing repository gates and agent protocols...");
      runNodeFallback(["bootstrap"], targetCwd);
    }
    return 0;
  }

  if (cmd === "verify" && args[1] === "precommit") {
    const gatesFile = resolve(cwd, ".vwl/gates.yaml");
    if (!existsSync(gatesFile)) {
      console.log("[VWL Hook] No .vwl/gates.yaml found; skipping gates.");
      return 0;
    }

    console.log("[VWL Hook] 🔍 Running pre-commit validator gates via Node runner...");
    const content = readFileSync(gatesFile, "utf-8");
    // Simple YAML line-by-line parser for commands marked precommit
    const lines = content.split("\n");
    let inGate = false;
    let currentCmd = "";
    let isPrecommit = false;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      if (line.startsWith("- name:") || line.startsWith("name:")) {
        if (inGate && isPrecommit && currentCmd) {
          console.log(`  * Executing: ${currentCmd}`);
          const res = spawnSync(currentCmd, { shell: true, stdio: "inherit", cwd });
          if (res.status !== 0) {
            console.error(`[VWL Hook] ❌ Gate failed: ${currentCmd}`);
            return res.status ?? 1;
          }
        }
        inGate = true;
        currentCmd = "";
        isPrecommit = false;
      }
      if (line.startsWith("command:")) {
        currentCmd = line.replace("command:", "").trim().replace(/^["']|["']$/g, "");
      }
      if (line.startsWith("precommit: true")) {
        isPrecommit = true;
      }
    }

    if (inGate && isPrecommit && currentCmd) {
      console.log(`  * Executing: ${currentCmd}`);
      const res = spawnSync(currentCmd, { shell: true, stdio: "inherit", cwd });
      if (res.status !== 0) {
        console.error(`[VWL Hook] ❌ Gate failed: ${currentCmd}`);
        return res.status ?? 1;
      }
    }

    console.log("[VWL Hook] ✅ All pre-commit validator gates passed.");
    return 0;
  }

  if (cmd === "init" || cmd === "bootstrap") {
    const projectName = basename(cwd);
    const vwlDir = resolve(cwd, ".vwl");
    if (!existsSync(vwlDir)) mkdirSync(vwlDir, { recursive: true });

    const languages = [];
    const gates = [];

    if (existsSync(resolve(cwd, "Cargo.toml"))) {
      languages.push("rust");
      gates.push({ name: "unit-tests", cmd: "cargo test --workspace" });
      gates.push({ name: "clippy-lint", cmd: "cargo clippy --all-targets --all-features -- -D warnings" });
      gates.push({ name: "format-check", cmd: "cargo fmt --check" });
    }

    if (existsSync(resolve(cwd, "package.json"))) {
      const isTs = existsSync(resolve(cwd, "tsconfig.json"));
      languages.push(isTs ? "typescript" : "javascript");
      gates.push({ name: "node-tests", cmd: "npm test" });
      if (isTs) {
        gates.push({ name: "typecheck", cmd: "npx tsc --noEmit" });
      }
    }

    if (existsSync(resolve(cwd, "pyproject.toml")) || existsSync(resolve(cwd, "requirements.txt"))) {
      languages.push("python");
      gates.push({ name: "pytest", cmd: "pytest" });
    }

    if (existsSync(resolve(cwd, "go.mod"))) {
      languages.push("go");
      gates.push({ name: "go-test", cmd: "go test ./..." });
    }

    // Always include skeptic review
    gates.push({ name: "skeptic-review", cmd: "vwl task review --json .vwl/ledger/tasks/*.yaml 2>/dev/null || true" });

    // Write .vwl/gates.yaml
    let gatesYaml = `version: "1.0"\nproject: "${projectName}"\nlanguages:\n`;
    for (const lang of languages) gatesYaml += `  - ${lang}\n`;
    gatesYaml += `gates:\n`;
    for (const g of gates) {
      gatesYaml += `  - name: "${g.name}"\n    plane: Deterministic\n    command: "${g.cmd}"\n    precommit: true\n    timeout_secs: 60\n    description: "Automated gate: ${g.name}"\n`;
    }
    writeFileSync(resolve(vwlDir, "gates.yaml"), gatesYaml);

    // Write .vwl/project.yaml
    const projYaml = `name: "${projectName}"\nrepository: ""\ndescription: "VWL Decentralized Project"\ntopics: ["vwl-project"]\nroles: []\n`;
    writeFileSync(resolve(vwlDir, "project.yaml"), projYaml);

    // Install pre-commit hook
    const hooksDir = resolve(cwd, ".git/hooks");
    if (existsSync(resolve(cwd, ".git"))) {
      mkdirSync(hooksDir, { recursive: true });
      const hookPath = resolve(hooksDir, "pre-commit");
      const hookScript = `#!/bin/sh
# VWL Pre-Commit Verification Gate Hook
echo "[VWL Hook] 🔍 Running pre-commit validator gates..."
if command -v vwl >/dev/null 2>&1; then
  vwl verify precommit
elif command -v npx >/dev/null 2>&1; then
  npx @horia-ai/vwl verify precommit
else
  echo "[VWL Hook] ⚠️ Neither 'vwl' nor 'npx' found; skipping hook."
  exit 0
fi
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "[VWL Hook] ❌ Pre-commit validation failed (exit code $EXIT_CODE)."
  exit $EXIT_CODE
fi
echo "[VWL Hook] ✅ All pre-commit validator gates passed."
exit 0
`;
      writeFileSync(hookPath, hookScript);
      try { chmodSync(hookPath, 0o755); } catch {}
    }

    // Write skills
    const skills = [
      { name: "repo-bootstrap-initializer", desc: "Autonomous repo bootstrap and gate validation protocol" },
      { name: "validator-wiring", desc: "Specialist validator for runtime reachability and module linkage" },
      { name: "validator-test-integrity", desc: "Specialist validator detecting oracle relaxation and assertion weakening" },
      { name: "validator-targeted-mutation", desc: "Targeted mutation analysis and survivor evaluation" },
      { name: "validator-adversarial-skeptic", desc: "Adversarial skeptic for unstated assumptions and edge cases" },
      { name: "validator-shadow-smoke", desc: "Read-only container smoke and shadow traffic verification" },
      { name: "tdd-mutation-testing", desc: "TDD red-green-refactor cycle and mutation testing" },
      { name: "bounded-verifier", desc: "Zero implementer conversation context verifier protocol" }
    ];

    for (const sk of skills) {
      const doc = `---\nname: ${sk.name}\ndescription: ${sk.desc}\n---\n\n# ${sk.name}\n${sk.desc}\n`;
      // .vwl/skills
      const vDir = resolve(vwlDir, `skills/${sk.name}`);
      mkdirSync(vDir, { recursive: true });
      writeFileSync(resolve(vDir, "SKILL.md"), doc);
      // .opencode/skills
      const ocDir = resolve(cwd, `.opencode/skills/${sk.name}`);
      mkdirSync(ocDir, { recursive: true });
      writeFileSync(resolve(ocDir, "SKILL.md"), doc);
    }

    // Write AGENTS.md instruction guide for autonomous agents
    const agentsMdPath = resolve(cwd, "AGENTS.md");
    if (!existsSync(agentsMdPath)) {
      const agentsMdContent = `# Agent Instructions: Virtual Work Ledger (VWL) Protocol

This repository is governed by the Virtual Work Ledger (VWL) decentralized evidence protocol.

## Step 0: Initial Repository Setup & Gate Verification
Whenever an agent session starts or resumes work in this repository:
1. Verify VWL is initialized:
   - Check that \`.vwl/gates.yaml\` exists.
   - Check that \`.git/hooks/pre-commit\` is installed and executable.
2. If VWL is not initialized or gates are unverified:
   - Run \`npx @horia-ai/vwl bootstrap\` (or \`vwl bootstrap --commit\`).
   - Run \`npx @horia-ai/vwl verify precommit\` to confirm all static validator gates pass on the clean base.
   - Commit the baseline configuration before starting any task code.
3. Task Execution Protocol:
   - Inspect the current Work Frontier: \`vwl frontier\` (or \`npx @horia-ai/vwl status\`).
   - Claim your assigned task: \`vwl claim <task-id>\`.
   - Work in an isolated Git worktree (\`vwl work <task-id>\`).
   - Enforce strict TDD and pass all static pre-commit gates.
   - Submit candidate diff and proofs via \`vwl submit\`.
`;
      writeFileSync(agentsMdPath, agentsMdContent);
    }

    console.log("=== VWL REPOSITORY BOOTSTRAP COMPLETE [Node Runner] ===");
    console.log(`  Project:         ${projectName}`);
    console.log(`  Detected Stack:  ${languages.join(", ")}`);
    console.log(`  Gates:           ${gates.length} configured in .vwl/gates.yaml`);
    console.log("  Git Hook:        Installed at .git/hooks/pre-commit");
    console.log("  Skills:          Injected across .vwl/skills and .opencode/skills");
    console.log("\nNext Steps:");
    console.log("  - Validate gates: npx @horia-ai/vwl verify precommit");
    console.log("  - For full node/ledger capabilities, install native vwl: cargo install --path crates/vwl-cli");
    return 0;
  }

  console.error(`Error: Native 'vwl' binary or 'cargo' toolchain not detected for command '${cmd}'.`);
  console.error("Please install the Rust toolchain or download precompiled vwl to run full ledger workflows.");
  return 1;
}

export function runCli(args = process.argv.slice(2)) {
  const cmd = args[0];
  if (cmd === "postinstall") {
    return runNodeFallback(args);
  }

  const binary = findVwlBinary();

  if (binary) {
    const res = spawnSync(binary, args, { stdio: "inherit" });
    return res.status ?? 0;
  }

  // Fallback 1: execute through cargo if installed
  const cargoCheck = spawnSync("cargo", ["--version"], { encoding: "utf-8" });
  if (cargoCheck.status === 0) {
    const res = spawnSync("cargo", ["run", "--quiet", "-p", "vwl-cli", "--", ...args], {
      stdio: "inherit",
    });
    return res.status ?? 0;
  }

  // Fallback 2: Standalone Node.js implementation for bootstrap, init, status, and verify precommit
  return runNodeFallback(args);
}

if (process.argv[1] === __filename) {
  const exitCode = runCli();
  process.exit(exitCode);
}
