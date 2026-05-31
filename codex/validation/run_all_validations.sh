#!/usr/bin/env bash
set -euo pipefail
REPO="${1:-.}"
python3 "$(dirname "$0")/run_all_validations.py" --repo "$REPO"
