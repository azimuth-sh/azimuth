package main

import "testing"

func TestIdentity(t *testing.T) {
	if identity() != "go" {
		t.Fatalf("expected go, got %s", identity())
	}
}
