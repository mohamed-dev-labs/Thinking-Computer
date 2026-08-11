"""Python/Hermes compatibility adapter for Thinking Computer.

This module deliberately owns no permissions, shell execution, session store, or
provider credentials. It forwards a typed local JSON request to the Rust binary,
which remains the authority for agent execution and user approval.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
import json
from pathlib import Path
import subprocess
from typing import Any, Mapping


@dataclass(frozen=True)
class AgentInput:
    """A normalized request that can be constructed from a Hermes callback."""

    prompt: str
    provider: str | None = None
    model: str | None = None
    session: str | None = None
    request_id: str | None = None


class ThinkingComputerEngine:
    """One-shot local client for ``thinking-computer rpc``.

    The adapter launches a child process without a shell, passes exactly one JSON
    request through standard input, and validates the JSON response. It never
    sends an input to a network endpoint itself.
    """

    def __init__(self, binary: str = "thinking-computer", config: Path | None = None) -> None:
        self._binary = binary
        self._config = config

    def run(self, request: AgentInput) -> Mapping[str, Any]:
        if not request.prompt.strip():
            raise ValueError("prompt must not be empty")
        command = [self._binary]
        if self._config is not None:
            command.extend(["--config", str(self._config)])
        command.append("rpc")
        payload = asdict(request)
        payload["id"] = payload.pop("request_id")
        completed = subprocess.run(
            command,
            input=json.dumps(payload) + "\n",
            capture_output=True,
            check=False,
            encoding="utf-8",
            timeout=600,
        )
        if completed.returncode != 0:
            raise RuntimeError(f"Rust engine failed: {completed.stderr.strip()}")
        try:
            response = json.loads(completed.stdout)
        except json.JSONDecodeError as error:
            raise RuntimeError("Rust engine returned invalid JSON") from error
        if not response.get("ok"):
            raise RuntimeError(str(response.get("error", "unknown engine error")))
        return response["result"]


def handle_hermes_input(event: Mapping[str, Any], engine: ThinkingComputerEngine | None = None) -> Mapping[str, Any]:
    """Forward a Hermes-compatible event to Rust.

    ``event`` may contain ``prompt``, ``provider``, ``model``, ``session``, and
    ``id``. The narrow contract means Hermes frontends and future Python input
    surfaces can share the same Rust policy boundary.
    """

    request = AgentInput(
        prompt=str(event.get("prompt", "")),
        provider=event.get("provider"),
        model=event.get("model"),
        session=event.get("session"),
        request_id=event.get("id"),
    )
    return (engine or ThinkingComputerEngine()).run(request)
