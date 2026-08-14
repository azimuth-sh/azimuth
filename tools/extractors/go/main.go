package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
)

type relation struct {
	Spec              string `json:"spec"`
	Scenario          string `json:"scenario"`
	Site              string `json:"site"`
	File              string `json:"file"`
	Lang              string `json:"lang"`
	SourceFingerprint string `json:"source_fingerprint"`
	Scope             string `json:"scope,omitempty"`
	Quantification    string `json:"quantification,omitempty"`
	Oracle            string `json:"oracle,omitempty"`
}

type mechanismImplementation struct {
	Spec              string `json:"spec"`
	Mechanism         string `json:"mechanism"`
	Binding           string `json:"binding"`
	File              string `json:"file"`
	Lang              string `json:"lang"`
	SourceFingerprint string `json:"source_fingerprint"`
}

type mechanismCover struct {
	Spec              string `json:"spec"`
	Mechanism         string `json:"mechanism"`
	Site              string `json:"site"`
	File              string `json:"file"`
	Lang              string `json:"lang"`
	SourceFingerprint string `json:"source_fingerprint"`
	Scope             string `json:"scope"`
	Quantification    string `json:"quantification"`
	Oracle            string `json:"oracle,omitempty"`
}

type artifact struct {
	ID   string `json:"id"`
	Kind string `json:"kind"`
	File string `json:"file"`
}

type manifest struct {
	Realizes                 []relation                `json:"realizes"`
	Covers                   []relation                `json:"covers"`
	MechanismImplementations []mechanismImplementation `json:"mechanism_implementations"`
	MechanismCovers          []mechanismCover          `json:"mechanism_covers"`
	ClassMembers             []any                     `json:"class_members"`
	Enumerations             []any                     `json:"enumerations"`
	Artifacts                []artifact                `json:"artifacts"`
}

func newManifest() manifest {
	return manifest{
		Realizes:                 []relation{},
		Covers:                   []relation{},
		MechanismImplementations: []mechanismImplementation{},
		MechanismCovers:          []mechanismCover{},
		ClassMembers:             []any{},
		Enumerations:             []any{},
		Artifacts:                []artifact{},
	}
}

func main() {
	output := flag.String("output", "", "manifest destination")
	root := flag.String("root", ".", "repository root")
	flag.Parse()
	if *output == "" || flag.NArg() == 0 {
		fmt.Fprintln(os.Stderr, "usage: azimuth-emit-go --output <path> [--root <dir>] <dir-or-file>...")
		os.Exit(2)
	}
	result, err := emit(flag.Args(), *root)
	if err != nil {
		fmt.Fprintf(os.Stderr, "azimuth-emit-go: %v\n", err)
		os.Exit(2)
	}
	encoded, err := json.MarshalIndent(result, "", "  ")
	if err != nil {
		panic(err)
	}
	if err := os.MkdirAll(filepath.Dir(*output), 0o755); err != nil {
		panic(err)
	}
	if err := os.WriteFile(*output, append(encoded, '\n'), 0o644); err != nil {
		panic(err)
	}
}

func emit(inputs []string, root string) (manifest, error) {
	result := newManifest()
	var files []string
	for _, input := range inputs {
		info, err := os.Stat(input)
		if err != nil {
			return result, err
		}
		if !info.IsDir() {
			files = append(files, input)
			continue
		}
		err = filepath.WalkDir(input, func(path string, entry os.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if entry.IsDir() && (entry.Name() == ".git" || entry.Name() == "vendor") {
				return filepath.SkipDir
			}
			if !entry.IsDir() && strings.HasSuffix(path, ".go") {
				files = append(files, path)
			}
			return nil
		})
		if err != nil {
			return result, err
		}
	}
	sort.Strings(files)
	for _, file := range files {
		if err := scanFile(file, root, &result); err != nil {
			return result, err
		}
	}
	return result, nil
}

func scanFile(path string, root string, result *manifest) error {
	absolutePath, err := filepath.Abs(path)
	if err != nil {
		return err
	}
	absoluteRoot, err := filepath.Abs(root)
	if err != nil {
		return err
	}
	source, err := os.ReadFile(absolutePath)
	if err != nil {
		return err
	}
	set := token.NewFileSet()
	parsed, err := parser.ParseFile(set, absolutePath, source, parser.SkipObjectResolution)
	if err != nil {
		return err
	}
	relative, err := filepath.Rel(absoluteRoot, absolutePath)
	if err != nil {
		return err
	}
	relative = filepath.ToSlash(relative)
	aliases, dotImport := markerImports(parsed)
	for _, declaration := range parsed.Decls {
		function, ok := declaration.(*ast.FuncDecl)
		if !ok || function.Body == nil {
			continue
		}
		start := set.Position(function.Pos()).Offset
		end := set.Position(function.End()).Offset
		fingerprint := sha256.Sum256(source[start:end])
		site := function.Name.Name
		ast.Inspect(function.Body, func(node ast.Node) bool {
			call, ok := node.(*ast.CallExpr)
			if !ok {
				return true
			}
			name := callName(call.Fun, aliases, dotImport)
			if !member(name, []string{"Realizes", "Covers", "ImplementsMechanism", "CoversMechanism"}) {
				return true
			}
			values, valueErr := stringArguments(call.Args)
			if valueErr != nil {
				err = fmt.Errorf("%s:%d: %s", relative, set.Position(call.Pos()).Line, valueErr)
				return false
			}
			valueErr = appendMarker(result, name, values, site, relative, hex.EncodeToString(fingerprint[:]))
			if valueErr != nil {
				err = fmt.Errorf("%s:%d: %s", relative, set.Position(call.Pos()).Line, valueErr)
				return false
			}
			return true
		})
		if err != nil {
			return err
		}
	}
	return nil
}

func markerImports(file *ast.File) (map[string]bool, bool) {
	aliases := map[string]bool{}
	dotImport := false
	for _, imported := range file.Imports {
		path, err := strconv.Unquote(imported.Path.Value)
		if err != nil || !strings.HasSuffix(path, "/azimuth") {
			continue
		}
		if imported.Name != nil {
			if imported.Name.Name == "." {
				dotImport = true
			} else if imported.Name.Name != "_" {
				aliases[imported.Name.Name] = true
			}
			continue
		}
		aliases[filepath.Base(path)] = true
	}
	return aliases, dotImport
}

func callName(expression ast.Expr, aliases map[string]bool, dotImport bool) string {
	switch value := expression.(type) {
	case *ast.Ident:
		if dotImport {
			return value.Name
		}
		return ""
	case *ast.SelectorExpr:
		qualifier, ok := value.X.(*ast.Ident)
		if ok && aliases[qualifier.Name] {
			return value.Sel.Name
		}
		return ""
	default:
		return ""
	}
}

func stringArguments(arguments []ast.Expr) ([]string, error) {
	values := make([]string, 0, len(arguments))
	for _, argument := range arguments {
		literal, ok := argument.(*ast.BasicLit)
		if !ok || literal.Kind != token.STRING {
			return nil, errors.New("marker arguments must be string literals")
		}
		values = append(values, strings.Trim(literal.Value, "`\""))
	}
	return values, nil
}

func appendMarker(result *manifest, name string, values []string, site string, file string, fingerprint string) error {
	minimum := map[string]int{"Realizes": 2, "Covers": 4, "ImplementsMechanism": 2, "CoversMechanism": 4}
	required, marker := minimum[name]
	if !marker {
		return nil
	}
	if len(values) < required {
		return fmt.Errorf("%s needs at least %d arguments", name, required)
	}
	if name == "Covers" || name == "CoversMechanism" {
		if err := validForm(values); err != nil {
			return err
		}
	}
	switch name {
	case "Realizes":
		result.Realizes = append(result.Realizes, relation{Spec: values[0], Scenario: values[1], Site: site, File: file, Lang: "go", SourceFingerprint: fingerprint})
	case "Covers":
		item := relation{Spec: values[0], Scenario: values[1], Site: site, File: file, Lang: "go", SourceFingerprint: fingerprint, Scope: values[2], Quantification: values[3]}
		if len(values) > 4 {
			item.Oracle = values[4]
		}
		result.Covers = append(result.Covers, item)
	case "ImplementsMechanism":
		binding := fmt.Sprintf("go-symbol:%s#%s", file, site)
		result.MechanismImplementations = append(result.MechanismImplementations, mechanismImplementation{Spec: values[0], Mechanism: values[1], Binding: binding, File: file, Lang: "go", SourceFingerprint: fingerprint})
		result.Artifacts = append(result.Artifacts, artifact{ID: binding, Kind: "go-symbol", File: file})
	case "CoversMechanism":
		item := mechanismCover{Spec: values[0], Mechanism: values[1], Site: site, File: file, Lang: "go", SourceFingerprint: fingerprint, Scope: values[2], Quantification: values[3]}
		if len(values) > 4 {
			item.Oracle = values[4]
		}
		result.MechanismCovers = append(result.MechanismCovers, item)
	}
	return nil
}

func validForm(values []string) error {
	if !member(values[2], []string{"unit", "component", "e2e"}) {
		return fmt.Errorf("unknown scope `%s`", values[2])
	}
	if !member(values[3], []string{"example", "universal"}) {
		return fmt.Errorf("unknown quantification `%s`", values[3])
	}
	if len(values) > 4 && !member(values[4], []string{"direct", "golden", "relational", "metamorphic", "model-based", "contract"}) {
		return fmt.Errorf("unknown oracle `%s`", values[4])
	}
	return nil
}

func member(value string, values []string) bool {
	for _, candidate := range values {
		if candidate == value {
			return true
		}
	}
	return false
}
