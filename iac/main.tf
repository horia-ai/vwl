terraform {
  required_version = ">= 1.5.0"
  required_providers {
    github = {
      source  = "integrations/github"
      version = "~> 6.0"
    }
  }
}

variable "owner" {
  type        = string
  description = "GitHub Organization owner"
  default     = "horia-ai"
}

variable "repository" {
  type        = string
  description = "GitHub repository name"
  default     = "vwl"
}

variable "github_token" {
  type        = string
  description = "GitHub Personal Access Token"
  sensitive   = true
  default     = ""
}

provider "github" {
  owner = var.owner
  token = var.github_token != "" ? var.github_token : null
}
