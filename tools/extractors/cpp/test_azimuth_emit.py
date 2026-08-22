import tempfile
import unittest
from pathlib import Path

from azimuth_emit import annotated_declarations, emit, scan


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

    def test_mechanism_uses_qualified_signature_and_path_free_companion(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            path = Path(directory) / "service.cpp"
            path.write_text(
                '#include "azimuth.hpp"\n'
                'namespace payments { struct Worker {\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("payments/capture", "completion-guard")\n'
                'int complete(const int & value) const { return value; }\n'
                '}; }\n',
                encoding="utf-8",
            )

            manifest = scan(path, self.repository, "clang++", self.include)
            implementation = manifest["mechanism_implementations"][0]
            self.assertEqual(
                set(implementation),
                {"spec", "mechanism", "site", "binding", "file", "lang", "source_fingerprint"},
            )
            self.assertEqual(
                implementation["site"],
                "payments::Worker::complete int (const int &) const",
            )
            self.assertEqual(
                implementation["binding"],
                f'cpp-symbol:{implementation["site"]}',
            )
            self.assertEqual(manifest["artifacts"][0]["id"], implementation["binding"])
            self.assertEqual(set(manifest["artifacts"][0]), {"id", "kind", "file"})
            self.assertNotIn(implementation["file"], implementation["binding"])

    def test_overloads_have_distinct_sites_and_relocation_is_stable(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            first = Path(directory) / "first.cpp"
            second = Path(directory) / "second.cpp"
            source = (
                '#include "azimuth.hpp"\nnamespace alpha {\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("alpha", "integer")\n'
                'int choose(int value) { return value; }\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("alpha", "floating")\n'
                'double choose(double value) { return value; }\n}\n'
            )
            first.write_text(source, encoding="utf-8")
            second.write_text(source, encoding="utf-8")

            before = scan(first, self.repository, "clang++", self.include)
            after = scan(second, self.repository, "clang++", self.include)
            sites = [item["site"] for item in before["mechanism_implementations"]]
            self.assertEqual(len(set(sites)), 2)
            self.assertIn("alpha::choose int (int)", sites)
            self.assertIn("alpha::choose double (double)", sites)
            self.assertEqual(
                [item["site"] for item in before["mechanism_implementations"]],
                [item["site"] for item in after["mechanism_implementations"]],
            )
            self.assertEqual(
                [item["source_fingerprint"] for item in before["mechanism_implementations"]],
                [item["source_fingerprint"] for item in after["mechanism_implementations"]],
            )
            for old, new in zip(
                before["mechanism_implementations"],
                after["mechanism_implementations"],
                strict=True,
            ):
                self.assertNotEqual(old["file"], new["file"])
                self.assertEqual(
                    {key: value for key, value in old.items() if key != "file"},
                    {key: value for key, value in new.items() if key != "file"},
                )

    def test_mechanism_marker_remains_two_arguments(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            path = Path(directory) / "service.cpp"
            path.write_text(
                '#include "azimuth.hpp"\n'
                '[[clang::annotate("azimuth|implements-mechanism|alpha")]]\n'
                'void run() {}\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "exactly two arguments"):
                scan(path, self.repository, "clang++", self.include)

    def test_one_declaration_retains_distinct_valid_annotations(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            path = Path(directory) / "service.cpp"
            path.write_text(
                '#include "azimuth.hpp"\n'
                'AZIMUTH_REALIZES("payments/capture", "completion-is-safe")\n'
                'AZIMUTH_IMPLEMENTS_CHECK("completion-check")\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("payments/capture", "completion-guard")\n'
                'void complete() {}\n',
                encoding="utf-8",
            )

            annotations = annotated_declarations(path, "clang++", self.include)
            manifest = scan(path, self.repository, "clang++", self.include)

            self.assertEqual(
                [annotation[2][1] for annotation in annotations],
                ["realizes", "implements-check", "implements-mechanism"],
            )
            self.assertEqual(len(manifest["realizes"]), 1)
            self.assertEqual(len(manifest["check_implementations"]), 1)
            self.assertEqual(len(manifest["mechanism_implementations"]), 1)
            fingerprints = {
                manifest["realizes"][0]["source_fingerprint"],
                manifest["check_implementations"][0]["source_fingerprint"],
                manifest["mechanism_implementations"][0]["source_fingerprint"],
            }
            self.assertEqual(len(fingerprints), 1)
            self.assertEqual(manifest["check_implementations"][0]["site"], "complete")
            self.assertEqual(
                manifest["mechanism_implementations"][0]["site"],
                "complete void ()",
            )

    def test_internal_anonymous_and_template_profiles_fail_closed(self) -> None:
        profiles = [
            ("internal linkage", (
                'AZIMUTH_IMPLEMENTS_MECHANISM("alpha", "guard")\n'
                'static void apply() {}\n'
            )),
            ("ambiguous annotated declaration", (
                'namespace {\nAZIMUTH_IMPLEMENTS_MECHANISM("alpha", "guard")\n'
                'void apply() {}\n}\n'
            )),
            ("templated or constrained", (
                'template <typename T>\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("alpha", "guard")\n'
                'void apply(T) {}\n'
            )),
            ("templated or constrained", (
                'template <typename T> requires true\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("alpha", "guard")\n'
                'void apply(T) {}\n'
            )),
        ]
        for message, declaration in profiles:
            with self.subTest(profile=message), tempfile.TemporaryDirectory(
                dir=self.repository
            ) as directory:
                path = Path(directory) / "service.cpp"
                path.write_text(
                    '#include "azimuth.hpp"\n' + declaration,
                    encoding="utf-8",
                )
                with self.assertRaisesRegex(ValueError, message):
                    scan(path, self.repository, "clang++", self.include)

    def test_source_locator_bearing_canonical_type_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            path = Path(directory) / "service.cpp"
            path.write_text(
                '#include "azimuth.hpp"\n'
                'using Local = decltype([] {});\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("alpha", "guard")\n'
                'void apply(Local) {}\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "contains a source locator"):
                scan(path, self.repository, "clang++", self.include)

    def test_alias_spelling_uses_clang_canonical_type(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            path = Path(directory) / "service.cpp"
            path.write_text(
                '#include "azimuth.hpp"\n'
                'namespace alpha { using Amount = int;\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("alpha", "guard")\n'
                'Amount apply(Amount value) { return value; }\n}\n',
                encoding="utf-8",
            )
            implementation = scan(path, self.repository, "clang++", self.include)[
                "mechanism_implementations"
            ][0]
            self.assertEqual(implementation["site"], "alpha::apply int (int)")

    def test_outside_root_fails_before_output(self) -> None:
        with tempfile.TemporaryDirectory() as root, tempfile.TemporaryDirectory() as outside:
            path = Path(outside) / "service.cpp"
            path.write_text("void ordinary() {}\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "outside --root"):
                emit([path], Path(root), "clang++", self.include)

    def test_header_annotation_is_not_attributed_to_including_source(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            root = Path(directory)
            header = root / "shared.hpp"
            header.write_text(
                '#include "azimuth.hpp"\n'
                'inline void ordinary() {}\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("alpha", "guard")\n'
                'inline void apply() {}\n',
                encoding="utf-8",
            )
            source = root / "service.cpp"
            source.write_text('#include "shared.hpp"\n', encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "source file"):
                emit([source], root, "clang++", [*self.include, root])

    def test_duplicate_site_across_translation_units_fails(self) -> None:
        with tempfile.TemporaryDirectory(dir=self.repository) as directory:
            root = Path(directory)
            first = root / "first.cpp"
            second = root / "second.cpp"
            declaration = (
                '#include "azimuth.hpp"\n'
                'AZIMUTH_IMPLEMENTS_MECHANISM("alpha", "{}")\n'
                'void apply() {{}}\n'
            )
            first.write_text(declaration.format("first"), encoding="utf-8")
            second.write_text(declaration.format("second"), encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "ambiguous mechanism site"):
                emit([root], root, "clang++", self.include)

if __name__ == "__main__":
    unittest.main()
