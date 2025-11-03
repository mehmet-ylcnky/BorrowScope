#!/bin/bash
# Generate coverage badge from JSON report

set -e

# Generate JSON coverage report
cargo llvm-cov --all-features --workspace --json --output-path coverage.json

# Extract coverage percentage
COVERAGE=$(jq -r '.data[0].totals.lines.percent' coverage.json | xargs printf "%.1f")

# Determine badge color
if (( $(echo "$COVERAGE >= 90" | bc -l) )); then
  COLOR="brightgreen"
elif (( $(echo "$COVERAGE >= 80" | bc -l) )); then
  COLOR="green"
elif (( $(echo "$COVERAGE >= 70" | bc -l) )); then
  COLOR="yellow"
else
  COLOR="red"
fi

echo "Coverage: ${COVERAGE}% (${COLOR})"

# Generate markdown table
echo ""
echo "## Coverage by Module"
echo ""
echo "| Module | Lines | Functions | Regions |"
echo "|--------|-------|-----------|---------|"

jq -r '.data[0].files[] | 
  select(.filename | contains("borrowscope-graph/src")) |
  "\(.filename | split("/") | last) | \(.summary.lines.percent | tonumber | round)% | \(.summary.functions.percent | tonumber | round)% | \(.summary.regions.percent | tonumber | round)%"' \
  coverage.json

echo ""
echo "**Overall: ${COVERAGE}% line coverage**"
