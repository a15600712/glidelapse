#!/bin/sh
# Block commits whose staged diff adds private data. Install: ln -sf ../../tools/pre-commit-privacy.sh .git/hooks/pre-commit
# ponytail: plain grep; swap for gitleaks if false positives/negatives show up.
PAT='([0-9]{1,3}\.){3}[0-9]{1,3}|[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[a-z]{2,}|/home/[a-z]|ssh [a-z0-9_-]+@|gh[pous]_[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]*PRIVATE KEY|xox[baprs]-|sk-[A-Za-z0-9]{20,}'
ALLOW='noreply@anthropic\.com|users\.noreply\.github\.com|lrtimelapse\.com|exiftool\.org|127\.0\.0\.1|0\.0\.0\.0|1\.0\.0|2\.0\.0'
hits=$(git diff --cached -U0 | grep -E '^\+' | grep -vE '^\+\+\+' | grep -nE "$PAT" | grep -vE "$ALLOW")
[ -z "$hits" ] && exit 0
echo "pre-commit: possible private data in staged changes:" >&2
echo "$hits" | cut -c1-160 >&2
echo "Remove it, or bypass once with: git commit --no-verify" >&2
exit 1
