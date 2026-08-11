#!/usr/bin/env python3
"""Fail closed when source-controlled files appear to contain credential material."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


SCANNED_SUFFIXES = {".c", ".cc", ".cpp", ".h", ".hpp", ".json", ".js", ".mjs", ".md", ".py", ".rs", ".toml", ".ts", ".tsx", ".yaml", ".yml"}
SKIPPED_PARTS = {".git", "target", "node_modules", ".venv", "__pycache__"}
KNOWN_TOKEN_RE = re.compile(r"\b(?:AKIA[0-9A-Z]{16}|AIza[0-9A-Za-z_-]{30,}|gh[pous]_[A-Za-z0-9_]{24,}|github_pat_[A-Za-z0-9_]{20,}|sk-[A-Za-z0-9_-]{20,}|xox[baprs]-[A-Za-z0-9-]{20,})\b")
PRIVATE_KEY_RE = re.compile(r"-----BEGIN(?: [A-Z]+)? PRIVATE KEY-----")
ASSIGNMENT_RE = re.compile(
    r"(?imx)^\s*(?:api[_-]?key|access[_-]?token|auth[_-]?token|client[_-]?secret|password|secret)\b\s*(?:=|:)\s*[\"']([^\"']{8,})[\"']"
)


def is_placeholder(value: str) -> bool:
    lowered = value.strip().casefold()
    if value.startswith("$") or re.fullmatch(r"[A-Z][A-Z0-9_]{2,}", value):
        return True
    return lowered in {"example", "replace-me", "your-key-here", "changeme", "redacted", "placeholder"}


def scan_file(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8", errors="ignore")
    findings: list[str] = []
    if KNOWN_TOKEN_RE.search(text):
        findings.append("known credential-shaped token")
    if PRIVATE_KEY_RE.search(text):
        findings.append("private-key header")
    if any(not is_placeholder(match.group(1)) for match in ASSIGNMENT_RE.finditer(text)):
        findings.append("literal credential assignment")
    return findings


def scan_repo(repo: Path) -> list[str]:
    findings: list[str] = []
    for path in repo.rglob("*"):
        if not path.is_file() or path.suffix not in SCANNED_SUFFIXES:
            continue
        if any(part in SKIPPED_PARTS for part in path.relative_to(repo).parts):
            continue
        for reason in scan_file(path):
            findings.append(f"{path.relative_to(repo)}: {reason}")
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Reject likely hard-coded credentials in source-controlled files")
    parser.add_argument("--repo", type=Path, required=True)
    args = parser.parse_args()
    findings = scan_repo(args.repo.resolve())
    if findings:
        print("security scan failed:", *findings, sep="\n- ", file=sys.stderr)
        return 1
    print("security scan passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
