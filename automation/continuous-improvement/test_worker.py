import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


MODULE_PATH = Path(__file__).with_name("run_improvement.py")
SPEC = importlib.util.spec_from_file_location("improvement_worker", MODULE_PATH)
worker = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(worker)


def plan_with_gate(command):
    return {
        "name": "test-plan",
        "version": 1,
        "slot_minutes": 60,
        "default_value_evidence": ["test coverage", "quality gates"],
        "quality_gates": [command],
        "tasks": [
            {"id": f"task-{number:02d}", "title": "test", "prompt": "review only"}
            for number in range(1, 21)
        ],
    }


class ImprovementWorkerTests(unittest.TestCase):
    def run_worker(self, plan):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".git").mkdir()
            plan_path = root / "plan.json"
            state_path = root / "state.json"
            plan_path.write_text(json.dumps(plan), encoding="utf-8")
            argv = [
                "run_improvement.py",
                "--repo",
                str(root),
                "--plan",
                str(plan_path),
                "--state",
                str(state_path),
                "--no-wait",
            ]
            with patch.object(sys, "argv", argv), patch.object(worker, "changed_files", return_value=[]):
                result = worker.main()
            return result, json.loads(state_path.read_text(encoding="utf-8"))

    def test_completes_all_slots_in_review_only_mode(self):
        result, state = self.run_worker(plan_with_gate([sys.executable, "-c", "pass"]))
        self.assertEqual(result, 0)
        self.assertEqual(len(state["completed"]), 20)
        self.assertEqual(len(state["reports"]), 20)
        self.assertEqual(state["reports"][0]["value_evidence"], ["test coverage", "quality gates"])
        self.assertEqual(
            state["reports"][0]["quality_gates"]["before_task"],
            [{"label": "quality-gate", "command": [sys.executable, "-c", "pass"]}],
        )
        self.assertEqual(
            state["reports"][0]["quality_gates"]["after_task"],
            [{"label": "quality-gate", "command": [sys.executable, "-c", "pass"]}],
        )
        self.assertFalse(state["halted"])

    def test_halts_and_persists_state_when_a_quality_gate_fails(self):
        result, state = self.run_worker(plan_with_gate([sys.executable, "-c", "raise SystemExit(2)"]))
        self.assertEqual(result, 1)
        self.assertTrue(state["halted"])
        self.assertEqual(state["completed"], [])
        self.assertEqual(state["events"][-1]["kind"], "task_halted")

    def test_halts_and_persists_state_when_a_security_gate_fails(self):
        failing_security_gate = {
            "label": "security-gate",
            "command": [sys.executable, "-c", "raise SystemExit(3)"],
        }
        result, state = self.run_worker(plan_with_gate(failing_security_gate))
        self.assertEqual(result, 1)
        self.assertTrue(state["halted"])
        self.assertEqual(state["completed"], [])
        command_event = next(event for event in state["events"] if event["kind"] == "command")
        self.assertEqual(command_event["label"], "security-gate")

    def test_rejects_change_sets_over_the_configured_limit(self):
        with patch.object(worker, "changed_files", return_value=["a.rs", "b.rs"]):
            with self.assertRaisesRegex(RuntimeError, "change limit exceeded"):
                worker.enforce_change_limit(Path("."), 1)

    def test_new_cycle_state_path_is_separate_from_resumable_default(self):
        path = worker.cycle_state_path(Path("/approved/repo"), "2026-08-11T21:00:00+00:00")
        self.assertEqual(path.parent.name, "improvement-cycles")
        self.assertTrue(path.name.endswith(".json"))
        self.assertNotEqual(path.name, "improvement-state.json")

    def test_rejects_plan_without_value_evidence(self):
        plan = plan_with_gate([sys.executable, "-c", "pass"])
        del plan["default_value_evidence"]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "plan.json"
            path.write_text(json.dumps(plan), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "default_value_evidence"):
                worker.load_plan(path)


if __name__ == "__main__":
    unittest.main()
