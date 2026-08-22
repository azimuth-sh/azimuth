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

    def test_mechanism_uses_module_and_qualname_without_file_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            first = Path(directory) / "first/pkg/capture.py"
            second = Path(directory) / "second/pkg/capture.py"
            first.parent.mkdir(parents=True)
            second.parent.mkdir(parents=True)
            (first.parent / "__init__.py").write_text("", encoding="utf-8")
            (second.parent / "__init__.py").write_text("", encoding="utf-8")
            source = (
                'class Worker:\n'
                '    @implements_mechanism("payments/capture", "completion-guard")\n'
                '    def complete(self, value):\n'
                '        return value\n'
            )
            first.write_text(source, encoding="utf-8")
            second.write_text(source, encoding="utf-8")

            before = scan(first, "first/pkg/capture.py", "pkg/capture.py")
            after = scan(second, "second/pkg/capture.py", "pkg/capture.py")
            implementation = before["mechanism_implementations"][0]

            self.assertEqual(
                set(implementation),
                {"spec", "mechanism", "site", "binding", "file", "lang", "source_fingerprint"},
            )
            self.assertEqual(implementation["site"], "pkg.capture.Worker.complete")
            self.assertEqual(
                implementation["binding"],
                "python-symbol:pkg.capture.Worker.complete",
            )
            self.assertEqual(before["artifacts"][0]["id"], implementation["binding"])
            self.assertEqual(before["artifacts"][0]["kind"], "python-symbol")
            self.assertEqual(set(before["artifacts"][0]), {"id", "kind", "file"})
            self.assertEqual(implementation["site"], after["mechanism_implementations"][0]["site"])
            self.assertEqual(
                implementation["source_fingerprint"],
                after["mechanism_implementations"][0]["source_fingerprint"],
            )
            self.assertNotEqual(
                implementation["file"],
                after["mechanism_implementations"][0]["file"],
            )
            before_without_file = {
                key: value for key, value in implementation.items() if key != "file"
            }
            after_without_file = {
                key: value
                for key, value in after["mechanism_implementations"][0].items()
                if key != "file"
            }
            self.assertEqual(before_without_file, after_without_file)

    def test_nested_qualnames_and_collisions_are_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            path.write_text(
                'def outer():\n'
                '    @implements_mechanism("alpha", "nested")\n'
                '    def run():\n'
                '        pass\n',
                encoding="utf-8",
            )
            implementation = scan(path, "pkg/service.py")["mechanism_implementations"][0]
            self.assertEqual(implementation["site"], "pkg.service.outer.<locals>.run")

            path.write_text(
                '@implements_mechanism("alpha", "first")\n'
                'def run():\n'
                '    pass\n\n'
                '@implements_mechanism("alpha", "second")\n'
                'def run():\n'
                '    pass\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "ambiguous mechanism site"):
                scan(path, "pkg/service.py")

    def test_mechanism_marker_remains_two_arguments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "service.py"
            path.write_text(
                '@implements_mechanism("alpha", "guard", "extra")\n'
                'def run():\n'
                '    pass\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "needs exactly 2"):
                scan(path, "service.py")

            path.write_text(
                '@implements_mechanism("alpha", "guard", unexpected="value")\n'
                'def run():\n'
                '    pass\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "does not accept keyword arguments"):
                scan(path, "service.py")

    def test_root_is_the_only_semantic_import_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source_root = Path(directory) / "src"
            source_root.mkdir()
            path = source_root / "service.py"
            path.write_text(
                '@implements_mechanism("alpha", "guard")\n'
                'def apply():\n'
                '    pass\n',
                encoding="utf-8",
            )
            implementation = emit([source_root], Path(directory))[
                "mechanism_implementations"
            ][0]
            self.assertEqual(implementation["site"], "src.service.apply")
            self.assertEqual(implementation["file"], "src/service.py")

            regrouped = emit([path], Path(directory))["mechanism_implementations"][0]
            self.assertEqual(implementation, regrouped)

    def test_outside_root_and_file_namespace_collisions_fail(self) -> None:
        with tempfile.TemporaryDirectory() as directory, tempfile.TemporaryDirectory() as outside:
            root = Path(directory)
            external = Path(outside) / "service.py"
            external.write_text("def ordinary(): pass\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "outside --root"):
                emit([external], root)

            module = root / "alpha.py"
            nested = root / "alpha/worker.py"
            nested.parent.mkdir()
            module.write_text("def ordinary(): pass\n", encoding="utf-8")
            nested.write_text("def ordinary(): pass\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "namespace `alpha` collides"):
                emit([root], root)

    def test_package_and_module_collision_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            module = root / "alpha.py"
            package = root / "alpha/__init__.py"
            package.parent.mkdir()
            module.write_text("def ordinary(): pass\n", encoding="utf-8")
            package.write_text("def ordinary(): pass\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "module `alpha` collides"):
                emit([root], root)


if __name__ == "__main__":
    unittest.main()
