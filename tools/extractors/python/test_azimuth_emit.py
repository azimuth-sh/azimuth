import tempfile
import unittest
from pathlib import Path

from azimuth_emit import emit, scan


class EmitterTests(unittest.TestCase):
    def test_decorators_resolve_the_enclosing_symbol_and_form(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            path.write_text(
                '@realizes("polyglot/identity", "python-identifies")\n'
                'def identity():\n    return "python"\n\n'
                '@covers("polyglot/identity", "python-identifies", "unit", "example", "direct")\n'
                'def test_identity():\n    assert identity() == "python"\n',
                encoding="utf-8",
            )

            manifest = scan(path, "service.py")

            self.assertEqual(manifest["realizes"][0]["site"], "identity")
            self.assertEqual(manifest["covers"][0]["scope"], "unit")
            self.assertEqual(manifest["covers"][0]["lang"], "python")

    def test_fingerprint_is_local_to_the_decorated_symbol(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            path.write_text('@realizes("a", "s")\ndef first():\n    return 1\n\ndef second():\n    return 2\n')
            before = scan(path, "service.py")["realizes"][0]["source_fingerprint"]
            path.write_text('@realizes("a", "s")\ndef first():\n    return 1\n\ndef second():\n    return 3\n')
            after = scan(path, "service.py")["realizes"][0]["source_fingerprint"]
            self.assertEqual(before, after)

    def test_invalid_form_fails_instead_of_dropping_the_tag(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            path.write_text('@covers("a", "s", "integration", "example")\ndef test_x():\n    pass\n')
            with self.assertRaisesRegex(ValueError, "unknown scope"):
                emit([path], Path(directory))


if __name__ == "__main__":
    unittest.main()
