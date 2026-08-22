package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"strings"
	"testing"
)

func writeSource(t *testing.T, directory string, source string) string {
	t.Helper()
	manifest := filepath.Join(directory, "go.mod")
	if _, err := os.Stat(manifest); os.IsNotExist(err) {
		_, current, _, ok := runtime.Caller(0)
		if !ok {
			t.Fatal("cannot locate Go extractor tests")
		}
		annotations, err := filepath.Abs(filepath.Join(filepath.Dir(current), "../../../packages/go"))
		if err != nil {
			t.Fatal(err)
		}
		module := fmt.Sprintf(
			"module example.test/service\n\ngo 1.24\n\n"+
				"require github.com/azimuth-sh/azimuth-go v0.0.0\n\n"+
				"replace github.com/azimuth-sh/azimuth-go => %s\n",
			filepath.ToSlash(annotations),
		)
		if err := os.WriteFile(manifest, []byte(module), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	path := filepath.Join(directory, "service.go")
	if err := os.WriteFile(path, []byte(source), 0o644); err != nil {
		t.Fatal(err)
	}
	return path
}

func TestMechanismUsesPackageReceiverAndTypeSignature(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import azimuth "github.com/azimuth-sh/azimuth-go/azimuth"
type Worker struct{}
func (worker *Worker) Apply(value string) error {
	azimuth.ImplementsMechanism("payments/capture", "completion-guard")
	return nil
}
func Apply(value int) int {
	azimuth.ImplementsMechanism("payments/capture", "free-guard")
	return value
}
`)

	result, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.MechanismImplementations) != 2 || len(result.Artifacts) != 2 {
		t.Fatalf("unexpected mechanism account: %#v %#v", result.MechanismImplementations, result.Artifacts)
	}
	sites := map[string]bool{}
	for index, implementation := range result.MechanismImplementations {
		sites[implementation.Site] = true
		if implementation.Binding != "go-symbol:"+implementation.Site {
			t.Fatalf("binding is not the typed site: %#v", implementation)
		}
		if strings.Contains(implementation.Binding, implementation.File) {
			t.Fatalf("binding contains the file locator: %#v", implementation)
		}
		if result.Artifacts[index].ID != implementation.Binding || result.Artifacts[index].Kind != "go-symbol" {
			t.Fatalf("companion does not match: %#v %#v", implementation, result.Artifacts[index])
		}
		encoded, marshalErr := json.Marshal(implementation)
		if marshalErr != nil {
			t.Fatal(marshalErr)
		}
		fields := map[string]any{}
		if err := json.Unmarshal(encoded, &fields); err != nil {
			t.Fatal(err)
		}
		if len(fields) != 7 || fields["site"] != implementation.Site {
			t.Fatalf("mechanism shape is not strict: %s", encoded)
		}
	}
	if !sites["example.test/service.(*Worker).Apply(string)->(error)"] ||
		!sites["example.test/service.Apply(int)->(int)"] {
		t.Fatalf("qualified sites were not preserved: %#v", sites)
	}
}

func TestMechanismRelocationIsStableAndDuplicateSiteFails(t *testing.T) {
	directory := t.TempDir()
	source := `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
type Worker struct{}
func (Worker) Apply(value string) { ImplementsMechanism("alpha", "guard") }
`
	first := writeSource(t, directory, source)
	before, err := emit([]string{first}, directory)
	if err != nil {
		t.Fatal(err)
	}
	second := filepath.Join(directory, "relocated.go")
	if err := os.Rename(first, second); err != nil {
		t.Fatal(err)
	}
	after, err := emit([]string{second}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if before.MechanismImplementations[0].Site != after.MechanismImplementations[0].Site ||
		before.MechanismImplementations[0].SourceFingerprint != after.MechanismImplementations[0].SourceFingerprint {
		t.Fatal("relocation changed semantic mechanism identity")
	}
	if before.MechanismImplementations[0].File == after.MechanismImplementations[0].File {
		t.Fatal("relocation did not change the accountable file locator")
	}

	first = writeSource(t, directory, source+`
func (Worker) Apply(value string) { ImplementsMechanism("alpha", "other") }
`)
	if _, err := emit([]string{first}, directory); err == nil {
		t.Fatal("expected the compiler account to reject a duplicate declaration")
	}
}

func TestMechanismMarkerRemainsTwoArguments(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
func Apply() { ImplementsMechanism("alpha", "guard", "extra") }
`)
	if _, err := emit([]string{path}, directory); err == nil || !strings.Contains(err.Error(), "needs exactly 2") {
		t.Fatalf("expected type-checked two-argument marker failure, got %v", err)
	}
}

func TestAnonymousMechanismSiteFailsClosed(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
func Apply() { callback := func() { ImplementsMechanism("alpha", "guard") }; callback() }
`)
	if _, err := emit([]string{path}, directory); err == nil ||
		!strings.Contains(err.Error(), "anonymous function has no stable Go site") {
		t.Fatalf("expected anonymous-site rejection, got %v", err)
	}
}

func TestCheckImplementationsResolveExactFunctions(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import azimuth "github.com/azimuth-sh/azimuth-go/azimuth"
func Identity() { azimuth.Realizes("polyglot/identity", "go-identifies") }
func TestFirst() { azimuth.ImplementsCheck("identity-check") }
func TestSecond() { azimuth.ImplementsCheck("identity-check"); println("second") }
func Unmarked() { println("unmarked") }
`)

	result, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.CheckImplementations) != 2 {
		t.Fatalf("unexpected implementations: %#v", result.CheckImplementations)
	}
	if result.CheckImplementations[0].Site != "TestFirst" || result.CheckImplementations[1].Site != "TestSecond" {
		t.Fatalf("unexpected sites: %#v", result.CheckImplementations)
	}
	pattern := regexp.MustCompile(`^sha256:[0-9a-f]{64}$`)
	for _, implementation := range result.CheckImplementations {
		if implementation.Check != "identity-check" || implementation.Lang != "go" ||
			!pattern.MatchString(implementation.SourceFingerprint) {
			t.Fatalf("unexpected implementation: %#v", implementation)
		}
	}
	if len(result.Realizes) != 1 || result.Realizes[0].Site != "Identity" {
		t.Fatalf("realization was not retained: %#v", result.Realizes)
	}
}

func TestFingerprintIsLocalToEachFunction(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
func First() { ImplementsCheck("check") }
func Second() { ImplementsCheck("check"); println("before") }
`)
	before, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	writeSource(t, directory, `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
func First() { ImplementsCheck("check") }
func Second() { ImplementsCheck("check"); println("after") }
`)
	after, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if before.CheckImplementations[0].SourceFingerprint != after.CheckImplementations[0].SourceFingerprint {
		t.Fatal("editing Second changed First's fingerprint")
	}
	if before.CheckImplementations[1].SourceFingerprint == after.CheckImplementations[1].SourceFingerprint {
		t.Fatal("editing Second did not change its fingerprint")
	}
}

func TestRetiredMarkersFailExplicitly(t *testing.T) {
	directory := t.TempDir()
	for _, marker := range []string{"Covers", "CoversMechanism"} {
		path := writeSource(t, directory, fmt.Sprintf(`package service
import azimuth "github.com/azimuth-sh/azimuth-go/azimuth"
		func Old() { azimuth.%s("a", "s", "unit", "example") }
`, marker))
		if _, err := emit([]string{path}, directory); err == nil || !strings.Contains(err.Error(), "retired alpha 1 marker "+marker) {
			t.Fatalf("expected explicit %s rejection, got %v", marker, err)
		}
	}
}

func TestUnrelatedCoversNameRemainsOrdinarySource(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
func Covers(value string) {}
func Unrelated() { Covers("check") }
`)
	result, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.CheckImplementations) != 0 {
		t.Fatalf("unrelated name was extracted: %#v", result.CheckImplementations)
	}
}

func TestUnrelatedMechanismHomonymIsNotPreflightedOrEmitted(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
func ImplementsMechanism(values ...string) {}
func Ordinary() { ImplementsMechanism("not", "an", "azimuth", "marker") }
`)
	result, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.MechanismImplementations) != 0 {
		t.Fatalf("unrelated mechanism homonym was extracted: %#v", result.MechanismImplementations)
	}
}

func TestImplementsCheckRequiresOneLiteral(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
func TestIdentity() { ImplementsCheck("a", "b") }
`)
	if _, err := emit([]string{path}, directory); err == nil {
		t.Fatal("expected invalid arity to fail")
	}
}

func TestGenericParametersArePositionalAcrossReceiverConstraintsAndUses(t *testing.T) {
	directory := t.TempDir()
	first := writeSource(t, directory, `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
type Box[T ~int] struct{}
func (Box[T]) Apply(value T) T { ImplementsMechanism("alpha", "receiver"); return value }
func Transform[U ~[]int](value U) U { ImplementsMechanism("alpha", "callable"); return value }
`)
	before, err := emit([]string{first}, directory)
	if err != nil {
		t.Fatal(err)
	}
	secondSource := `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
type Box[Element ~int] struct{}
func (Box[Element]) Apply(value Element) Element { ImplementsMechanism("alpha", "receiver"); return value }
func Transform[Values ~[]int](value Values) Values { ImplementsMechanism("alpha", "callable"); return value }
`
	second := writeSource(t, directory, secondSource)
	after, err := emit([]string{second}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if len(before.MechanismImplementations) != 2 || len(after.MechanismImplementations) != 2 {
		t.Fatalf("unexpected generic mechanisms: %#v %#v", before.MechanismImplementations, after.MechanismImplementations)
	}
	for index := range before.MechanismImplementations {
		if before.MechanismImplementations[index].Site != after.MechanismImplementations[index].Site {
			t.Fatalf("generic spelling leaked into site: %q != %q", before.MechanismImplementations[index].Site, after.MechanismImplementations[index].Site)
		}
	}
	account := before.MechanismImplementations[0].Site + before.MechanismImplementations[1].Site
	if !strings.Contains(account, "Box[$0]") ||
		!strings.Contains(account, "$0:~int") ||
		!strings.Contains(account, "$0:~[]int") {
		t.Fatalf("generic positions or constraints are absent: %q", account)
	}
}

func TestGenericConstraintChangesSemanticSite(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
func Apply[T ~int](value T) T { ImplementsMechanism("alpha", "guard"); return value }
`)
	before, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	writeSource(t, directory, `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
func Apply[T ~string](value T) T { ImplementsMechanism("alpha", "guard"); return value }
`)
	after, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if before.MechanismImplementations[0].Site == after.MechanismImplementations[0].Site {
		t.Fatal("generic constraint change did not change semantic site")
	}
}

func TestOutsideRootFailsClosed(t *testing.T) {
	root := t.TempDir()
	outside := t.TempDir()
	path := writeSource(t, outside, "package service\nfunc Ordinary() {}\n")
	if _, err := emit([]string{path}, root); err == nil || !strings.Contains(err.Error(), "outside --root") {
		t.Fatalf("expected outside-root rejection, got %v", err)
	}
}

func TestInputGroupingDoesNotChangePackageIdentity(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import . "github.com/azimuth-sh/azimuth-go/azimuth"
func Apply(value int) int { ImplementsMechanism("alpha", "guard"); return value }
`)
	byFile, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	byRoot, err := emit([]string{directory, path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if fmt.Sprint(byFile) != fmt.Sprint(byRoot) {
		t.Fatalf("input grouping changed manifest: %#v %#v", byFile, byRoot)
	}
}

func TestCompilerRejectsInvalidMarkedFunctionBody(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import azimuth "github.com/azimuth-sh/azimuth-go/azimuth"
func Apply(value int) int {
	azimuth.ImplementsMechanism("alpha", "guard")
	return missing
}
`)
	if _, err := emit([]string{path}, directory); err == nil ||
		!strings.Contains(err.Error(), "Go compiler account failed") {
		t.Fatalf("expected full compiler rejection, got %v", err)
	}
}

func TestCompilerAccountLoadsStandardLibraryExports(t *testing.T) {
	directory := t.TempDir()
	path := writeSource(t, directory, `package service
import (
	"net/http"
	azimuth "github.com/azimuth-sh/azimuth-go/azimuth"
)
func Apply(request *http.Request) string {
	azimuth.ImplementsMechanism("alpha", "guard")
	return request.Method
}
`)
	result, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.MechanismImplementations) != 1 ||
		!strings.Contains(result.MechanismImplementations[0].Site, "*net/http.Request") {
		t.Fatalf("standard-library type identity was not loaded: %#v", result.MechanismImplementations)
	}
}

func TestDuplicateSiteAcrossPackagesFailsClosed(t *testing.T) {
	root := t.TempDir()
	first := filepath.Join(root, "first")
	second := filepath.Join(root, "second")
	if err := os.MkdirAll(first, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.MkdirAll(second, 0o755); err != nil {
		t.Fatal(err)
	}
	firstPath := writeSource(t, first, `package service
import azimuth "github.com/azimuth-sh/azimuth-go/azimuth"
func Apply() { azimuth.ImplementsMechanism("alpha", "first") }
`)
	secondPath := writeSource(t, second, `package service
import azimuth "github.com/azimuth-sh/azimuth-go/azimuth"
func Apply() { azimuth.ImplementsMechanism("alpha", "second") }
`)
	if _, err := emit([]string{firstPath, secondPath}, root); err == nil ||
		!strings.Contains(err.Error(), "ambiguous mechanism site") {
		t.Fatalf("expected cross-package collision rejection, got %v", err)
	}
}
