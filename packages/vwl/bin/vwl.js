#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
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

export function runCli(args = process.argv.slice(2)) {
  const binary = findVwlBinary();

  if (binary) {
    const res = spawnSync(binary, args, { stdio: "inherit" });
    return res.status ?? 0;
  }

  // Fallback: execute through cargo if installed
  const cargoCheck = spawnSync("cargo", ["--version"], { encoding: "utf-8" });
  if (cargoCheck.status === 0) {
    const res = spawnSync("cargo", ["run", "--quiet", "-p", "vwl-cli", "--", ...args], {
      stdio: "inherit",
    });
    return res.status ?? 0;
  }

  console.error("Error: Could not locate native 'vwl' binary or 'cargo' toolchain.");
  console.error("Please ensure the Rust toolchain is installed or download the precompiled vwl binary.");
  return 1;
}

if (process.argv[1] === __filename) {
  const exitCode = runCli();
  process.exit(exitCode);
}
