package main

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"
)

func writeSource(t *testing.T, directory string, source string) string {
	t.Helper()
	path := filepath.Join(directory, "service.go")
	if err := os.WriteFile(path, []byte(source), 0o644); err != nil {
		t.Fatal(err)
	}
	return path
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
