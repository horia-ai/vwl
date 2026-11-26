# VWL Infrastructure as Code (IaC)

This directory manages the GitHub organization and repository infrastructure for **Virtual Work Ledger (VWL)** declaratively as code.

## 🏗️ Managed Infrastructure

1. **GitHub Teams** ([`teams.tf`](file:///Users/trifan/DEVELOPMENT/vwl/iac/teams.tf)):
   - `vwl-maintainers`: Lead architects & maintainers (`admin` permission on `vwl`)
   - `vwl-developers`: Developer agents & implementers (`push` permission on `vwl`)
   - `vwl-verifiers`: Independent verifiers & auditors (`triage` permission on `vwl`)
   - `vwl-tokenomics`: Economic modeling, simulator researchers (`push` permission on `vwl`)
   - `vwl-contracts`: Solidity smart contract engineers (`push` permission on `vwl`)

2. **GitHub Labels** ([`labels.tf`](file:///Users/trifan/DEVELOPMENT/vwl/iac/labels.tf)):
   - Gate labels: `gate:0` through `gate:10`
   - Type labels: `type:work-unit`, `type:gate-decision`, `type:infrastructure`

3. **GitHub Project V2 Board**:
   - Board: [VWL Development & Gate Delivery](https://github.com/orgs/horia-ai/projects/1)
   - Synchronized via [`scripts/sync_project_board.py`](file:///Users/trifan/DEVELOPMENT/vwl/iac/scripts/sync_project_board.py)

4. **Gate Milestones & Work Units**:
   - Milestones for Gates 0 to 10
   - Declarative issue tracking and synchronization

---

## 🚀 Usage

### Requirements
- Terraform `>= 1.5.0`
- GitHub Personal Access Token with `admin:org` and `repo` scopes (`GITHUB_TOKEN` or `ATRIFAN_GITHUB_PAT_TOKEN`)

### Plan & Apply
```bash
# Initialize Terraform
cd iac
terraform init

# Plan changes
export GITHUB_TOKEN="<your_pat_token>"
terraform plan

# Apply changes
terraform apply -auto-approve

# Synchronize Project Board
python3 scripts/sync_project_board.py
```

### Automation
The CI workflow [`.github/workflows/iac.yml`](file:///Users/trifan/DEVELOPMENT/vwl/.github/workflows/iac.yml) automatically runs `terraform fmt` and `terraform plan` on pull requests to ensure zero configuration drift.
