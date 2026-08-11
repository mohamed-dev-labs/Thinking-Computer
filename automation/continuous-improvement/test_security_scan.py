import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("security_scan.py")
SPEC = importlib.util.spec_from_file_location("improvement_security_scan", MODULE_PATH)
scanner = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(scanner)


class SecurityScanTests(unittest.TestCase):
    def test_allows_environment_variable_references(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "config.toml").write_text('api_key = "OPENAI_API_KEY"\n', encoding="utf-8")
            self.assertEqual(scanner.scan_repo(root), [])

    def test_flags_literal_credential_assignment(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "config.toml").write_text('api_key = "unredacted-credential-123456"\n', encoding="utf-8")
            self.assertTrue(any("literal credential assignment" in finding for finding in scanner.scan_repo(root)))


if __name__ == "__main__":
    unittest.main()
