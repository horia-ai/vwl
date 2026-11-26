locals {
  labels = {
    "gate:0"              = { color = "0E8A16", description = "Gate 0 - Minimum Falsifiable VWL" }
    "gate:1"              = { color = "1D76DB", description = "Gate 1 - Node and Project Identity" }
    "gate:2"              = { color = "5319E7", description = "Gate 2 - GitHub Discovery & Market" }
    "gate:3"              = { color = "0052CC", description = "Gate 3 - CLI and TUI" }
    "gate:4"              = { color = "FBCA04", description = "Gate 4 - Reputation and Evidence" }
    "gate:5"              = { color = "F9D0C4", description = "Gate 5 - Local Economic State Machine" }
    "gate:6"              = { color = "D93F0B", description = "Gate 6 - Economic Simulator" }
    "gate:7"              = { color = "B60205", description = "Gate 7 - Testnet Contracts" }
    "gate:8"              = { color = "7057FF", description = "Gate 8 - Wallet and Chain TUI" }
    "gate:9"              = { color = "006B75", description = "Gate 9 - Fiat Onramp & Stripe" }
    "gate:10"             = { color = "C2E0C6", description = "Gate 10 - Productive Transfer & DEX" }
    "type:work-unit"      = { color = "BFD4F2", description = "Bounded Work Unit" }
    "type:gate-decision"  = { color = "E99695", description = "Gate Exit Decision" }
    "type:infrastructure" = { color = "C5DEF5", description = "CI/CD & Repo Tooling" }
  }
}

resource "github_issue_label" "labels" {
  for_each    = local.labels
  repository  = var.repository
  name        = each.key
  color       = each.value.color
  description = each.value.description
}
