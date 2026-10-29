#!/usr/bin/env bash

# ramcourt_snapshot.sh: print a compact traffic summary and append it to a log
# at the moment this script is meant for maintainer-only and requires `gh auth login` with push access.

set -euo pipefail
R="OferMania/ramcourt-draft"
LOG="${1:-$HOME/ramcourt-traffic.log}"
{
  echo "$(date +%F) (GitHub keeps 14 days)"
  gh api repos/$R/traffic/views  --jq '"Views \(.count) / \(.uniques) unique; peak day \(.views | max_by(.count) | .timestamp[0:10])"'
  gh api repos/$R/traffic/clones --jq '"Clones \(.count) / \(.uniques) unique; peak day \(.clones | max_by(.count) | .timestamp[0:10])"'
  gh api repos/$R/traffic/popular/referrers --jq '"Referrers: " + (map("\(.referrer) \(.count)") | join(", "))'
  gh api repos/$R/traffic/popular/paths     --jq '"Top pages: " + (map("\(.path) (\(.count)/\(.uniques))") | join(", "))'
  gh api repos/$R --jq '"Stars: \(.stargazers_count)  Forks: \(.forks_count)"'
  echo "Replies elsewhere: (add by hand)"
  echo
  echo "Daily Views (last 14 days):"
  gh api repos/$R/traffic/views --jq '.views[] | "\(.timestamp[0:10]) \(.count) \(.uniques)"'
  echo
  echo "Daily Clones (last 14 days):"
  gh api repos/$R/traffic/clones --jq '.clones[] | "\(.timestamp[0:10]) \(.count) \(.uniques)"'
  echo
  echo "Discussions (last 20):"
  gh api graphql -f query=' { repository(owner:"OferMania", name:"ramcourt-draft") { discussions(first: 20) { nodes { number title comments { totalCount } } } } }' \
  --jq '.data.repository.discussions.nodes[] | "discussion \(.number): \(.comments.totalCount) comments"'
  echo
} | tee -a "$LOG"
