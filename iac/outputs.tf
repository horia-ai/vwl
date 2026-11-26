output "team_slugs" {
  description = "Slugs of all created VWL teams"
  value       = { for k, v in github_team.teams : k => v.slug }
}

output "label_names" {
  description = "Names of all created issue labels"
  value       = [for k, v in github_issue_label.labels : v.name]
}
