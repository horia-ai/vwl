import { describe, it, expect } from "vitest";
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
});
