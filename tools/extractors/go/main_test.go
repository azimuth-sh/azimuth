package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestCompilerTreeResolvesFunctionAndForm(t *testing.T) {
	directory := t.TempDir()
	path := filepath.Join(directory, "service.go")
	source := `package service
import azimuth "github.com/drim-dev/azimuth-go/azimuth"
func Identity() { azimuth.Realizes("polyglot/identity", "go-identifies") }
func TestIdentity() { azimuth.Covers("polyglot/identity", "go-identifies", "unit", "example", "direct") }
`
	if err := os.WriteFile(path, []byte(source), 0o644); err != nil {
		t.Fatal(err)
	}

	result, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if result.Realizes[0].Site != "Identity" || result.Realizes[0].Lang != "go" {
		t.Fatalf("unexpected realization: %#v", result.Realizes[0])
	}
	if result.Covers[0].Scope != "unit" || result.Covers[0].Oracle != "direct" {
		t.Fatalf("unexpected cover: %#v", result.Covers[0])
	}
}

func TestInvalidFormFailsClosed(t *testing.T) {
	directory := t.TempDir()
	path := filepath.Join(directory, "service.go")
	os.WriteFile(path, []byte(`package service
import . "github.com/drim-dev/azimuth-go/azimuth"
func TestIdentity() { Covers("a", "s", "integration", "example") }
`), 0o644)

	_, err := emit([]string{path}, directory)
	if err == nil {
		t.Fatal("expected invalid scope to fail")
	}
}

func TestIgnoresUnrelatedCallsWithTheSameNames(t *testing.T) {
	directory := t.TempDir()
	path := filepath.Join(directory, "service.go")
	os.WriteFile(path, []byte(`package service
func Realizes(spec string, scenario string) {}
func Identity() { Realizes("polyglot/identity", "go-identifies") }
`), 0o644)

	result, err := emit([]string{path}, directory)
	if err != nil {
		t.Fatal(err)
	}
	if len(result.Realizes) != 0 {
		t.Fatalf("unrelated call was extracted: %#v", result.Realizes)
	}
}
