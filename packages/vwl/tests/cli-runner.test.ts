import { describe, it, expect } from "vitest";
import { findVwlBinary, runCli } from "../bin/vwl.js";

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
});
