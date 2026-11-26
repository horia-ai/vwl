locals {
  teams = {
    "vwl-maintainers" = {
      description = "Lead architects, maintainers, and coordinators for VWL protocol"
      privacy     = "closed"
      permission  = "admin"
    }
    "vwl-developers" = {
      description = "Developer agents and software engineers contributing bounded Work Units"
      privacy     = "closed"
      permission  = "push"
    }
    "vwl-verifiers" = {
      description = "Independent verification agents, reviewers, and security auditors"
      privacy     = "closed"
      permission  = "triage"
    }
    "vwl-tokenomics" = {
      description = "Economic modeling, discrete-event simulator research, and game theory"
      privacy     = "closed"
      permission  = "push"
    }
    "vwl-contracts" = {
      description = "Solidity smart contract engineers and testnet deployment specialists"
      privacy     = "closed"
      permission  = "push"
    }
  }
}

resource "github_team" "teams" {
  for_each    = local.teams
  name        = each.key
  description = each.value.description
  privacy     = each.value.privacy
}

resource "github_team_repository" "team_repo" {
  for_each   = local.teams
  team_id    = github_team.teams[each.key].id
  repository = var.repository
  permission = each.value.permission
}
