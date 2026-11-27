# Publishing `@horia-ai/vwl` to NPM & Sccache Acceleration Guide

This document outlines the end-to-end configuration for publishing `@horia-ai/vwl` to the public NPM Registry (`npmjs.com`), managing authentication secrets, and leveraging Mozilla `sccache` with the GitHub Actions Cache backend for zero-external-cloud compilation caching.

---

## 1. NPM Credentials & Token Management

### GitHub Actions Secrets
The repository contains the GitHub Actions secret:
* **`NPM_TOKEN`**: Configured in GitHub Repository Secrets (`Settings -> Secrets and variables -> Actions`).
* In GitHub Actions workflows (`.github/workflows/publish-package.yml`), `npm publish` reads:
  ```yaml
  env:
    NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
  ```

### Local Development Environment
The automation token is also stored in your local shell profile (`~/.zprofile`):
```bash
export ATRIFAN_NPM_TOKEN="npm_***"
```
To publish directly from your local terminal:
```bash
cd packages/vwl
NODE_AUTH_TOKEN="$ATRIFAN_NPM_TOKEN" npm publish --access public
```

---

## 2. Automated Publishing via GitHub Actions & Enterprise OIDC (Option B)

The package publishing pipeline is automated in [`.github/workflows/publish-package.yml`](../.github/workflows/publish-package.yml).

### Why Option B (Enterprise Grade) is Recommended

| Feature | Option A: Classic Token | Option B: OIDC Trusted Publishing (Enterprise) |
|---|---|---|
| **Secret Management** | Long-lived static secret `NPM_TOKEN` (leak/rotation risk) | **Zero Secrets** (ephemeral short-lived GitHub OIDC tokens) |
| **Supply Chain Security** | No cryptographic build origin proof | **SLSA Provenance via Sigstore** (tamper-proof proof of commit & repo) |
| **Auditability** | Package uploaded from arbitrary runner with token | Cryptographically verified public ledger linking npm tarball to GitHub SHA |
| **Compliance** | Vulnerable to credential interception | Enterprise SOC2/ISO-27001 supply chain compliance |

### Configuring Option B (OIDC Trusted Publishing) on npmjs.com

1. Navigate to **[npmjs.com](https://www.npmjs.com)** -> Package **`@horia-ai/vwl`** (or create organization `@horia-ai`).
2. Go to **Settings** -> **Publishing Access** -> **Trusted Publishers**.
3. Click **Add Publisher** -> Select **GitHub Actions**:
   * **GitHub organization/user**: `horia-ai`
   * **Repository name**: `vwl`
   * **Workflow filename**: `publish-package.yml`
   * **Environment**: *(leave blank or specify production)*
4. Save. Once configured, GitHub Actions can publish with `--provenance` even without any static `NPM_TOKEN`!

### Triggering a Release

1. **Dry-Run Validation (Validate build, sccache, and tests without publishing):**
   ```bash
   env GH_TOKEN="$ATRIFAN_GITHUB_PAT_TOKEN" gh workflow run publish-package.yml -f dry_run=true
   ```
2. **Pushing a Git Release Tag:**
   ```bash
   git tag v0.1.1
   git push origin v0.1.1
   ```
3. **Publishing a GitHub Release:**
   Creating and publishing a Release from GitHub UI triggers the production build and publish workflow.
4. **Manual Workflow Dispatch:**
   ```bash
   env GH_TOKEN="$ATRIFAN_GITHUB_PAT_TOKEN" gh workflow run publish-package.yml
   ```

### Workflow Steps
1. Checks out the repository.
2. Initializes the Rust toolchain and Mozilla `sccache` with GitHub Actions Cache backend.
3. Compiles the native release binary `vwl-cli`.
4. Runs `npm test` and `cargo test --all`.
5. Publishes `@horia-ai/vwl` to `https://registry.npmjs.org/` with public visibility.

---

## 3. Rust Build Caching with `sccache` (Zero External Cloud Accounts)

To ensure rapid build times without incurring third-party cloud storage fees (AWS S3, GCP, or Azure), VWL uses **Mozilla `sccache` backed by the GitHub Actions Cache API**.

### How It Works
* **Action:** `mozilla-actions/sccache-action@v0.0.11` (with `actions/github-script@v7` cache environment bridge)
* **Configuration:**
  ```yaml
  - name: Expose GitHub Actions Cache Environment
    uses: actions/github-script@v7
    with:
      script: |
        core.exportVariable('ACTIONS_CACHE_URL', process.env.ACTIONS_CACHE_URL || process.env.ACTIONS_RESULTS_URL || '');
        core.exportVariable('ACTIONS_RUNTIME_TOKEN', process.env.ACTIONS_RUNTIME_TOKEN || '');
        core.exportVariable('ACTIONS_CACHE_SERVICE_V2', process.env.ACTIONS_CACHE_SERVICE_V2 || 'true');

  - name: Set up sccache
    uses: mozilla-actions/sccache-action@v0.0.11

  - name: Build Rust binary
    env:
      SCCACHE_GHA_ENABLED: "true"
      RUSTC_WRAPPER: "sccache"
    run: cargo build --release -p vwl-cli
  ```
* **Storage:** Leverages GitHub's built-in 10 GB Actions Cache quota directly through `ACTIONS_CACHE_URL` and `ACTIONS_RUNTIME_TOKEN`.
* **Zero External Services:** No AWS, GCP, or Cloudflare credentials needed.
* **Benefits:**
  - Identical crate compilation units are shared across jobs.
  - Release compilation drops from ~25s down to <5s on warm cache.

### Local Development Build Caching

You can use the exact same `sccache` engine locally on your development machine to accelerate every `cargo build`, `cargo test`, and `cargo clippy`:

1. **Install `sccache` via Homebrew:**
   ```bash
   brew install sccache
   ```

2. **Configure your shell profile (`~/.zprofile` or `~/.zshrc`):**
   ```bash
   if command -v /opt/homebrew/bin/sccache >/dev/null 2>&1 || command -v sccache >/dev/null 2>&1; then
     export RUSTC_WRAPPER="sccache"
     export SCCACHE_DIR="$HOME/Library/Caches/Mozilla.sccache"
     export SCCACHE_CACHE_SIZE="10G"
   fi
   ```

3. **Verify cache utilization:**
   ```bash
   sccache --show-stats
   ```
   * Warm rebuild read hits execute in **~0.015s per crate** (sub-second incremental and clean rebuilds).

---

## 4. End-User Installation & NPX Execution (Option B: Enterprise Grade)

### What Makes Option B "Enterprise Grade"?
1. **Zero Prerequisites**: Works out of the box on machines without Rust, Cargo, or system toolchains installed.
2. **Autonomous Stack Detection**: Identifies whether the target repository is TypeScript/JavaScript, Rust, Python, Go, or mixed, and configures matching deterministic static validator gates in `.vwl/gates.yaml`.
3. **Automated Baseline Verification**: Runs each generated gate immediately to ensure green state before any configuration is committed.
4. **Deterministic Pre-Commit Hook**: Installs `.git/hooks/pre-commit` to prevent unvalidated agent or human commits from entering the Git tree.
5. **Universal Agent Harness Injection**: Automatically injects verifier skills into `.vwl/skills`, `.opencode/skills`, `.claude/skills`, `.gemini/skills`, and `.github/skills`.
6. **Cryptographic Identity Provisioning**: Generates decentralized Ed25519 node keypair in `.vwl/credentials/node_key.json` for task claiming and proof signing.

### Commands

```bash
# Instant zero-dependency enterprise bootstrap in any repository
npx @horia-ai/vwl bootstrap

# Auto-stage and commit initial bootstrap baseline
npx @horia-ai/vwl bootstrap --commit

# Inspect VWL status and local ledger state
npx @horia-ai/vwl status

# Run pre-commit validator gates
npx @horia-ai/vwl verify precommit
```

And for high-throughput cryptographic replay and economic settlement loops:
```bash
cargo install --git https://github.com/horia-ai/vwl.git vwl-cli
# or once on crates.io:
cargo install vwl
```
