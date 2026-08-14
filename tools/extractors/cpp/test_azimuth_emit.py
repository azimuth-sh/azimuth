import tempfile
import unittest
from pathlib import Path

from azimuth_emit import scan


class EmitterTests(unittest.TestCase):
    def test_clang_annotations_bind_to_compiled_functions(self) -> None:
        repository = Path(__file__).resolve().parents[3]
        with tempfile.TemporaryDirectory(dir=repository) as directory:
            path = Path(directory) / "service.cpp"
            path.write_text(
                '#include "azimuth.hpp"\n'
                'AZIMUTH_REALIZES("polyglot/identity", "cpp-identifies")\n'
                'const char* identity() { return "cpp"; }\n'
                'AZIMUTH_COVERS("polyglot/identity", "cpp-identifies", "unit", "example", "direct")\n'
                'void identity_test() {}\n',
                encoding="utf-8",
            )

            manifest = scan(path, repository, "clang++", [repository / "packages/cpp"])

            self.assertEqual(manifest["realizes"][0]["site"], "identity")
            self.assertEqual(manifest["covers"][0]["scope"], "unit")
            self.assertEqual(manifest["covers"][0]["lang"], "cpp")


if __name__ == "__main__":
    unittest.main()
