#!/usr/bin/env python3
"""Run a bounded, resumable Thinking Computer improvement plan on an isolated VM.

The worker deliberately starts in review-only mode. `--execute-agent` enables calls to a
locally configured Thinking Computer binary and requires `--vm-sentinel` as an explicit
acknowledgement that edits occur inside a disposable or approved VM.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def load_plan(path: Path) -> dict[str, Any]:
    plan = json.loads(path.read_text(encoding="utf-8"))
    if plan.get("version") != 1 or not isinstance(plan.get("tasks"), list):
        raise ValueError("unsupported or malformed improvement plan")
    if len(plan["tasks"]) != 20:
        raise ValueError("the continuous plan must contain exactly 20 hourly tasks")
    return plan


def load_state(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {"completed": [], "events": [], "reports": [], "halted": False}
    state = json.loads(path.read_text(encoding="utf-8"))
    state.setdefault("completed", [])
    state.setdefault("events", [])
    state.setdefault("reports", [])
    state.setdefault("halted", False)
    return state


def save_state(path: Path, state: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def event(state: dict[str, Any], kind: str, **details: Any) -> None:
    state["events"].append({"at": utc_now(), "kind": kind, **details})


def run_checked(command: list[str], repo: Path, state: dict[str, Any], label: str) -> None:
    result = subprocess.run(command, cwd=repo, text=True, capture_output=True, check=False)
    event(state, "command", label=label, command=command, returncode=result.returncode,
          stdout=result.stdout[-4000:], stderr=result.stderr[-4000:])
    if result.returncode != 0:
        raise RuntimeError(f"quality gate failed: {label}")


def quality_gates(plan: dict[str, Any], repo: Path, state: dict[str, Any]) -> list[dict[str, Any]]:
    successful_commands: list[dict[str, Any]] = []
    for gate in plan["quality_gates"]:
        if isinstance(gate, dict):
            command = gate.get("command")
            label = gate.get("label", "quality-gate")
        else:
            command = gate
            label = "quality-gate"
        if not isinstance(command, list) or not all(isinstance(item, str) for item in command):
            raise ValueError("every quality gate must contain a string command list")
        if not isinstance(label, str):
            raise ValueError("every quality gate label must be a string")
        run_checked(command, repo, state, label)
        successful_commands.append({"label": label, "command": command})
    return successful_commands


def working_tree_changed(repo: Path) -> bool:
    result = subprocess.run(["git", "status", "--porcelain"], cwd=repo, text=True,
                            capture_output=True, check=True)
    return bool(result.stdout.strip())


def changed_files(repo: Path) -> list[str]:
    result = subprocess.run(["git", "status", "--porcelain"], cwd=repo, text=True,
                            capture_output=True, check=True)
    return [line[3:] for line in result.stdout.splitlines() if len(line) >= 4]


def enforce_change_limit(repo: Path, limit: int) -> list[str]:
    files = changed_files(repo)
    if len(files) > limit:
        raise RuntimeError(f"change limit exceeded: {len(files)} files is greater than {limit}")
    return files


def invoke_agent(args: argparse.Namespace, task: dict[str, Any], repo: Path, state: dict[str, Any]) -> None:
    if not args.execute_agent:
        event(state, "review_only", task=task["id"], prompt=task["prompt"])
        return
    sentinel = repo / args.vm_sentinel
    if not sentinel.exists():
        raise RuntimeError(f"VM sentinel is required before autonomous edits: {sentinel}")
    command = [args.agent_binary, "--workspace", str(repo), "--yes", "chat"]
    if args.provider:
        command.extend(["--provider", args.provider])
    command.append(task["prompt"])
    run_checked(command, repo, state, f"agent:{task['id']}")


def commit_if_requested(args: argparse.Namespace, task: dict[str, Any], repo: Path, state: dict[str, Any]) -> None:
    if not args.commit or not working_tree_changed(repo):
        return
    if len(state["completed"]) >= args.max_commits:
        raise RuntimeError("maximum meaningful commit count reached")
    run_checked(["git", "add", "-A"], repo, state, "git-add")
    run_checked(["git", "commit", "-m", f"improve: {task['title']}"], repo, state, "git-commit")


def main() -> int:
    parser = argparse.ArgumentParser(description="Run the Thinking Computer 20-hour improvement plan")
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--plan", type=Path, default=Path(__file__).with_name("20-hour-plan.json"))
    parser.add_argument("--state", type=Path)
    parser.add_argument("--duration-hours", type=int, default=20)
    parser.add_argument("--execute-agent", action="store_true")
    parser.add_argument("--agent-binary", default="thinking-computer")
    parser.add_argument("--provider")
    parser.add_argument("--vm-sentinel", default=".thinking-computer/VM_ONLY")
    parser.add_argument("--commit", action="store_true")
    parser.add_argument("--max-commits", type=int, default=20)
    parser.add_argument("--max-changed-files", type=int, default=30)
    parser.add_argument("--no-wait", action="store_true", help="run slots consecutively for controlled testing")
    args = parser.parse_args()

    repo = args.repo.resolve()
    if not (repo / ".git").exists():
        raise SystemExit("--repo must point to a Git working tree")
    if args.duration_hours != 20:
        raise SystemExit("this bounded plan is intentionally fixed at 20 hours")
    if not 1 <= args.max_changed_files <= 100:
        raise SystemExit("--max-changed-files must be between 1 and 100")
    plan = load_plan(args.plan)
    state_path = args.state or repo / ".thinking-computer" / "improvement-state.json"
    state = load_state(state_path)
    if state.get("halted"):
        raise SystemExit("worker is halted; inspect the audit log and clear state only after review")

    completed = set(state["completed"])
    for task in plan["tasks"]:
        if task["id"] in completed:
            continue
        slot_started = time.monotonic()
        event(state, "task_started", task=task["id"], title=task["title"])
        try:
            pre_task_gates = quality_gates(plan, repo, state)
            invoke_agent(args, task, repo, state)
            files = enforce_change_limit(repo, args.max_changed_files)
            post_task_gates = quality_gates(plan, repo, state)
            commit_if_requested(args, task, repo, state)
        except Exception as error:  # preserve state and stop instead of continuing after a failed gate
            event(state, "task_halted", task=task["id"], error=str(error))
            state["halted"] = True
            save_state(state_path, state)
            print(f"HALTED: {error}", file=sys.stderr)
            return 1
        state["completed"].append(task["id"])
        state["reports"].append({
            "task": task["id"],
            "title": task["title"],
            "rationale": task["prompt"],
            "quality_gates": {
                "before_task": pre_task_gates,
                "after_task": post_task_gates,
            },
            "changed_files": files,
            "residual_risk": task.get(
                "residual_risk",
                "Review the task-linked diff and local audit events before enabling production use.",
            ),
        })
        event(state, "task_completed", task=task["id"])
        save_state(state_path, state)
        remaining = plan["slot_minutes"] * 60 - (time.monotonic() - slot_started)
        if remaining > 0 and not args.no_wait:
            time.sleep(remaining)
    print(json.dumps({"status": "completed", "state": str(state_path), "tasks": len(state["completed"])}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
