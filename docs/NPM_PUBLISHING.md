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

## 2. Automated Publishing via GitHub Actions

The package publishing pipeline is automated in [`.github/workflows/publish-package.yml`](../.github/workflows/publish-package.yml).

### Triggering a Release
Publishing is triggered automatically upon:
1. **Pushing a Git Release Tag:**
   ```bash
   git tag v0.1.1
   git push origin v0.1.1
   ```
2. **Publishing a GitHub Release:**
   Creating and publishing a Release from the GitHub UI triggers the build and publish workflow.
3. **Manual Workflow Dispatch:**
   You can manually dispatch the workflow via GitHub CLI:
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
* **Action:** `mozilla-actions/sccache-action@v0.0.8` (with `actions/github-script@v7` cache environment bridge)
* **Configuration:**
  ```yaml
  - name: Expose GitHub Actions Cache Environment
    uses: actions/github-script@v7
    with:
      script: |
        core.exportVariable('ACTIONS_CACHE_URL', process.env.ACTIONS_CACHE_URL || process.env.ACTIONS_RESULTS_URL || '');
        core.exportVariable('ACTIONS_RUNTIME_TOKEN', process.env.ACTIONS_RUNTIME_TOKEN || '');

  - name: Set up sccache
    uses: mozilla-actions/sccache-action@v0.0.8
    with:
      version: "v0.8.2"

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

---

## 4. End-User Installation & NPX Execution

Once published to `npmjs.com`, any developer or agent can run:

```bash
# Instant zero-dependency bootstrap in any repository
npx @horia-ai/vwl bootstrap

# Inspect VWL status
npx @horia-ai/vwl status

# Run pre-commit validator gates
npx @horia-ai/vwl verify precommit
```

And for high-throughput cryptographic and economic operations, native installation:
```bash
cargo install --git https://github.com/horia-ai/vwl.git vwl-cli
# or once on crates.io:
cargo install vwl
```
