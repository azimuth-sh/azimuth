import tempfile
import unittest
from pathlib import Path

from azimuth_emit import emit, scan


class EmitterTests(unittest.TestCase):
    def test_check_implementations_resolve_exact_symbols(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            path.write_text(
                '@realizes("polyglot/identity", "python-identifies")\n'
                'def identity():\n    return "python"\n\n'
                '@implements_check("identity-check")\n'
                'def test_first():\n    assert identity() == "python"\n\n'
                '@implements_check("identity-check")\n'
                'def test_second():\n    assert identity()\n\n'
                'def unmarked():\n    pass\n',
                encoding="utf-8",
            )

            manifest = scan(path, "service.py")

            self.assertEqual(manifest["realizes"][0]["site"], "identity")
            implementations = manifest["check_implementations"]
            self.assertEqual([item["site"] for item in implementations], ["test_first", "test_second"])
            for item in implementations:
                self.assertEqual(item["check"], "identity-check")
                self.assertEqual(item["lang"], "python")
                self.assertRegex(item["source_fingerprint"], r"^sha256:[0-9a-f]{64}$")
                self.assertNotIn("spec", item)

    def test_fingerprint_is_local_to_each_implementation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            path.write_text(
                '@implements_check("check")\ndef first():\n    return 1\n\n'
                '@implements_check("check")\ndef second():\n    return 2\n'
            )
            before = scan(path, "service.py")["check_implementations"]
            path.write_text(
                '@implements_check("check")\ndef first():\n    return 1\n\n'
                '@implements_check("check")\ndef second():\n    return 3\n'
            )
            after = scan(path, "service.py")["check_implementations"]
            self.assertEqual(before[0]["source_fingerprint"], after[0]["source_fingerprint"])
            self.assertNotEqual(before[1]["source_fingerprint"], after[1]["source_fingerprint"])

    def test_retired_decorators_fail_explicitly(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            for marker in ("covers", "covers_mechanism"):
                path.write_text(f'@{marker}("a", "s")\ndef old():\n    pass\n')
                with self.assertRaisesRegex(
                    ValueError, f"retired alpha 1 marker {marker} is not supported"
                ):
                    scan(path, "service.py")

    def test_unrelated_qualified_covers_decorator_remains_ordinary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            path.write_text(
                'def covers(_case):\n'
                '    return lambda target: target\n\n'
                '@covers("case")\n'
                '@helpers.covers("case")\n'
                'def ordinary():\n'
                '    pass\n'
            )
            self.assertEqual(scan(path, "service.py")["check_implementations"], [])

    def test_implements_check_requires_one_literal(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            path.write_text('@implements_check("a", "b")\ndef test_x():\n    pass\n')
            with self.assertRaisesRegex(ValueError, "needs exactly 1"):
                emit([path], Path(directory))


if __name__ == "__main__":
    unittest.main()
