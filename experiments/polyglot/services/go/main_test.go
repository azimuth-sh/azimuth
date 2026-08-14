package main

import (
	"testing"

	azimuth "github.com/drim-dev/azimuth-go/azimuth"
)

func TestIdentity(t *testing.T) {
	azimuth.Covers("polyglot/identity", "go-identifies", "unit", "example", "direct")
	if identity() != "go" {
		t.Fatalf("expected go, got %s", identity())
	}
}
