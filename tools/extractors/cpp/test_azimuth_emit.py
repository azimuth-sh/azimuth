import tempfile
import unittest
from pathlib import Path

from azimuth_emit import scan


class EmitterTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repository = Path(__file__).resolve().parents[3]
        self.include = [self.repository / "packages/cpp"]

    def test_check_implementations_bind_to_exact_functions(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            path = Path(directory) / "service.cpp"
            path.write_text(
                '#include "azimuth.hpp"\n'
                'AZIMUTH_REALIZES("polyglot/identity", "cpp-identifies")\n'
                'const char* identity() { return "cpp"; }\n'
                'AZIMUTH_IMPLEMENTS_CHECK("identity-check")\n'
                'void first_test() {}\n'
                'AZIMUTH_IMPLEMENTS_CHECK("identity-check")\n'
                'void second_test() { (void)identity(); }\n'
                'void unmarked() {}\n',
                encoding="utf-8",
            )

            manifest = scan(path, self.repository, "clang++", self.include)

            self.assertEqual(manifest["realizes"][0]["site"], "identity")
            implementations = manifest["check_implementations"]
            self.assertEqual([item["site"] for item in implementations], ["first_test", "second_test"])
            for item in implementations:
                self.assertEqual(item["check"], "identity-check")
                self.assertEqual(item["lang"], "cpp")
                self.assertRegex(item["source_fingerprint"], r"^sha256:[0-9a-f]{64}$")
                self.assertNotIn("spec", item)

    def test_fingerprint_is_local_to_each_function(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            path = Path(directory) / "service.cpp"
            path.write_text(
                '#include "azimuth.hpp"\n'
                'AZIMUTH_IMPLEMENTS_CHECK("check")\nvoid first() {}\n'
                'AZIMUTH_IMPLEMENTS_CHECK("check")\nvoid second() { int value = 1; }\n'
            )
            before = scan(path, self.repository, "clang++", self.include)["check_implementations"]
            path.write_text(
                '#include "azimuth.hpp"\n'
                'AZIMUTH_IMPLEMENTS_CHECK("check")\nvoid first() {}\n'
                'AZIMUTH_IMPLEMENTS_CHECK("check")\nvoid second() { int value = 2; }\n'
            )
            after = scan(path, self.repository, "clang++", self.include)["check_implementations"]
            self.assertEqual(before[0]["source_fingerprint"], after[0]["source_fingerprint"])
            self.assertNotEqual(before[1]["source_fingerprint"], after[1]["source_fingerprint"])

    def test_old_macro_is_absent(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            path = Path(directory) / "service.cpp"
            path.write_text(
                '#include "azimuth.hpp"\n'
                'AZIMUTH_COVERS("a", "s", "unit", "example", "direct")\nvoid old() {}\n'
            )
            with self.assertRaisesRegex(ValueError, "type specifier|required"):
                scan(path, self.repository, "clang++", self.include)


if __name__ == "__main__":
    unittest.main()
