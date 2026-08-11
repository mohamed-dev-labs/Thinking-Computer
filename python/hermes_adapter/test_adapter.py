import json
import unittest
from types import SimpleNamespace
from unittest.mock import patch

from thinking_computer_adapter import AgentInput, ThinkingComputerEngine, handle_hermes_input


class HermesAdapterTest(unittest.TestCase):
    @patch("thinking_computer_adapter.subprocess.run")
    def test_forwards_a_single_typed_request_to_rust_rpc(self, run):
        run.return_value = SimpleNamespace(returncode=0, stderr="", stdout=json.dumps({"ok": True, "result": {"text": "done"}}))
        result = ThinkingComputerEngine(binary="tc-test").run(
            AgentInput(prompt="inspect workspace", provider="ollama", request_id="req-1")
        )
        self.assertEqual(result["text"], "done")
        command = run.call_args.args[0]
        self.assertEqual(command, ["tc-test", "rpc"])
        payload = json.loads(run.call_args.kwargs["input"])
        self.assertEqual(payload["id"], "req-1")
        self.assertEqual(payload["prompt"], "inspect workspace")

    @patch("thinking_computer_adapter.ThinkingComputerEngine.run")
    def test_maps_a_hermes_style_event_to_agent_input(self, run):
        run.return_value = {"text": "ok"}
        result = handle_hermes_input({"id": "event-1", "prompt": "hello"})
        self.assertEqual(result, {"text": "ok"})
        forwarded = run.call_args.args[0]
        self.assertEqual(forwarded.request_id, "event-1")


if __name__ == "__main__":
    unittest.main()
