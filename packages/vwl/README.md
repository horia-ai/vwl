# @horia-ai/vwl

Official npm distribution package for the **Virtual Work Ledger (VWL) CLI**.

## 🚀 Quick Usage via NPX

Run any VWL command immediately without manual installation:

```bash
# Initialize VWL in your repo (detects languages, sets up .vwl/gates.yaml & pre-commit hook)
npx @horia-ai/vwl init

# Run pre-commit validator gates
npx @horia-ai/vwl verify precommit

# Check repository ledger status
npx @horia-ai/vwl status

# Replay canonical state
npx @horia-ai/vwl replay

# Generate contributor effectiveness & cost allocation report
npx @horia-ai/vwl report

# Install skills into target agent harnesses
npx @horia-ai/vwl skill install tdd-mutation-testing --harness all
```

## 📦 Global Installation

```bash
# Install globally
npm install -g @horia-ai/vwl

# Execute directly
vwl status
vwl report --gh-pages
```
