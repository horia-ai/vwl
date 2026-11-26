import os
import json
import time
import urllib.request
import urllib.error

TOKEN = os.environ.get("GITHUB_TOKEN") or os.environ.get("ATRIFAN_GITHUB_PAT_TOKEN")
REPO = "horia-ai/vwl"
PROJECT_ID = "PVT_kwDOFRLXoM4BtZXB"  # Project 1: VWL Development & Gate Delivery

HEADERS = {
    "Authorization": f"token {TOKEN}",
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "VWL-IaC-Sync"
}

def graphql_query(query, variables=None):
    url = "https://api.github.com/graphql"
    data = json.dumps({"query": query, "variables": variables or {}}).encode("utf-8")
    req = urllib.request.Request(url, data=data, headers=HEADERS, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        print(f"GraphQL HTTP Error {e.code}: {e.read().decode('utf-8')}")
        raise

def get_repo_issues():
    url = f"https://api.github.com/repos/{REPO}/issues?state=all&per_page=100"
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode("utf-8"))

def sync_issues_to_project():
    print(f"Fetching issues for {REPO}...")
    issues = get_repo_issues()
    print(f"Found {len(issues)} issues. Adding to ProjectV2 ({PROJECT_ID})...")

    mutation = """
    mutation AddItem($projectId: ID!, $contentId: ID!) {
      addProjectV2ItemById(input: { projectId: $projectId, contentId: $contentId }) {
        item {
          id
        }
      }
    }
    """

    added = 0
    for issue in issues:
        # Skip pull requests if any returned by issues API
        if "pull_request" in issue:
            continue
        content_id = issue["node_id"]
        issue_num = issue["number"]
        issue_title = issue["title"]

        try:
            res = graphql_query(mutation, {"projectId": PROJECT_ID, "contentId": content_id})
            if "errors" in res:
                print(f"  Warning on issue #{issue_num}: {res['errors']}")
            else:
                item_id = res["data"]["addProjectV2ItemById"]["item"]["id"]
                added += 1
                if added % 10 == 0 or added == len(issues):
                    print(f"  Synced {added}/{len(issues)} issues to Project Board (latest: #{issue_num})...")
            time.sleep(0.2)
        except Exception as e:
            print(f"  Error adding issue #{issue_num}: {e}")

    print(f"Successfully synced {added} issues to GitHub ProjectV2 Board!")

if __name__ == "__main__":
    sync_issues_to_project()
