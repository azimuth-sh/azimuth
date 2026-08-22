package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"unicode"
)

const annotationPackage = "github.com/azimuth-sh/azimuth-go/azimuth"

type relation struct {
	Spec              string `json:"spec"`
	Scenario          string `json:"scenario"`
	Site              string `json:"site"`
	File              string `json:"file"`
	Lang              string `json:"lang"`
	SourceFingerprint string `json:"source_fingerprint"`
}

type checkImplementation struct {
	Check             string `json:"check"`
	Site              string `json:"site"`
	File              string `json:"file"`
	Lang              string `json:"lang"`
	SourceFingerprint string `json:"source_fingerprint"`
}

type mechanismImplementation struct {
	Spec              string `json:"spec"`
	Mechanism         string `json:"mechanism"`
	Site              string `json:"site"`
	Binding           string `json:"binding"`
	File              string `json:"file"`
	Lang              string `json:"lang"`
	SourceFingerprint string `json:"source_fingerprint"`
}

type artifact struct {
	ID   string `json:"id"`
	Kind string `json:"kind"`
	File string `json:"file"`
}

type manifest struct {
	Realizes                 []relation                `json:"realizes"`
	CheckImplementations     []checkImplementation     `json:"check_implementations"`
	MechanismImplementations []mechanismImplementation `json:"mechanism_implementations"`
	ClassMembers             []any                     `json:"class_members"`
	Enumerations             []any                     `json:"enumerations"`
	Artifacts                []artifact                `json:"artifacts"`
}

func newManifest() manifest {
	return manifest{
		Realizes:                 []relation{},
		CheckImplementations:     []checkImplementation{},
		MechanismImplementations: []mechanismImplementation{},
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
	absoluteRoot, err := canonicalPath(root)
	if err != nil {
		return result, fmt.Errorf("%s: cannot resolve --root: %w", root, err)
	}
	rootInfo, err := os.Stat(absoluteRoot)
	if err != nil || !rootInfo.IsDir() {
		return result, fmt.Errorf("%s: --root must be a directory", root)
	}
	var files []string
	for _, input := range inputs {
		selected, resolveErr := canonicalPath(input)
		if resolveErr != nil {
			return result, fmt.Errorf("%s: cannot resolve input: %w", input, resolveErr)
		}
		if selected != absoluteRoot {
			if _, withinErr := workspaceRelative(absoluteRoot, selected); withinErr != nil {
				return result, withinErr
			}
		}
		info, err := os.Stat(selected)
		if err != nil {
			return result, err
		}
		if !info.IsDir() {
			if filepath.Ext(selected) != ".go" {
				return result, fmt.Errorf("%s: Go input must be a .go file or directory", input)
			}
			files = append(files, selected)
			continue
		}
		err = filepath.WalkDir(selected, func(path string, entry os.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if entry.IsDir() && (entry.Name() == ".git" || entry.Name() == "vendor") {
				return filepath.SkipDir
			}
			if !entry.IsDir() && strings.HasSuffix(path, ".go") {
				resolved, resolveErr := canonicalPath(path)
				if resolveErr != nil {
					return resolveErr
				}
				if _, withinErr := workspaceRelative(absoluteRoot, resolved); withinErr != nil {
					return withinErr
				}
				files = append(files, resolved)
			}
			return nil
		})
		if err != nil {
			return result, err
		}
	}
	sort.Strings(files)
	files = uniqueStrings(files)
	packages := map[string][]string{}
	for _, file := range files {
		packages[filepath.Dir(file)] = append(packages[filepath.Dir(file)], file)
	}
	directories := make([]string, 0, len(packages))
	for directory := range packages {
		directories = append(directories, directory)
	}
	sort.Strings(directories)
	for _, directory := range directories {
		if err := scanPackage(directory, packages[directory], absoluteRoot, &result); err != nil {
			return result, err
		}
	}
	if err := validateMechanismSites(result.MechanismImplementations); err != nil {
		return result, err
	}
	sortManifest(&result)
	return result, nil
}

func canonicalPath(value string) (string, error) {
	absolute, err := filepath.Abs(value)
	if err != nil {
		return "", err
	}
	return filepath.EvalSymlinks(filepath.Clean(absolute))
}

func workspaceRelative(root string, path string) (string, error) {
	relative, err := filepath.Rel(root, path)
	if err != nil || relative == "." || relative == ".." || strings.HasPrefix(relative, ".."+string(filepath.Separator)) {
		return "", fmt.Errorf("%s: input is outside --root", path)
	}
	normalized := filepath.ToSlash(relative)
	if normalized == "" || strings.Contains(normalized, "\\") {
		return "", fmt.Errorf("%s: file is not a normalized workspace-relative path", path)
	}
	for _, segment := range strings.Split(normalized, "/") {
		if segment == "" || segment == "." || segment == ".." {
			return "", fmt.Errorf("%s: file is not a normalized workspace-relative path", path)
		}
	}
	return normalized, nil
}

func uniqueStrings(values []string) []string {
	if len(values) == 0 {
		return values
	}
	result := []string{values[0]}
	for _, value := range values[1:] {
		if value != result[len(result)-1] {
			result = append(result, value)
		}
	}
	return result
}

func scanPackage(directory string, selected []string, root string, result *manifest) error {
	absoluteDirectory, err := filepath.Abs(directory)
	if err != nil {
		return err
	}
	absoluteRoot := root
	packagePath, err := packageImportPath(absoluteDirectory)
	if err != nil {
		return err
	}
	set := token.NewFileSet()
	entries, err := os.ReadDir(absoluteDirectory)
	if err != nil {
		return err
	}
	parsedByPath := map[string]*ast.File{}
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".go") {
			continue
		}
		path := filepath.Join(absoluteDirectory, entry.Name())
		parsed, parseErr := parser.ParseFile(set, path, nil, parser.SkipObjectResolution)
		if parseErr != nil {
			return parseErr
		}
		parsedByPath[path] = parsed
	}
	if len(parsedByPath) == 0 {
		return fmt.Errorf("%s: no Go source files", directory)
	}
	var packageName string
	packageFiles := make([]*ast.File, 0, len(parsedByPath))
	for _, parsed := range parsedByPath {
		if packageName == "" {
			packageName = parsed.Name.Name
		}
		if parsed.Name.Name != packageName {
			return fmt.Errorf("%s: multiple package names prevent one semantic identity", directory)
		}
		packageFiles = append(packageFiles, parsed)
	}
	selectedSet := map[string]bool{}
	for _, path := range selected {
		absolute, absoluteErr := filepath.Abs(path)
		if absoluteErr != nil {
			return absoluteErr
		}
		selectedSet[absolute] = true
	}
	if err := preflightMarkerCalls(parsedByPath, selectedSet, set); err != nil {
		return err
	}
	info := &types.Info{
		Defs: map[*ast.Ident]types.Object{},
		Uses: map[*ast.Ident]types.Object{},
	}
	compilerImporter, err := packageImporter(absoluteDirectory, set)
	if err != nil {
		return err
	}
	configuration := &types.Config{Importer: compilerImporter}
	if _, checkErr := configuration.Check(packagePath, set, packageFiles, info); checkErr != nil {
		return fmt.Errorf("%s: Go compiler rejected package: %w", directory, checkErr)
	}
	mechanismSites := map[string][2]string{}
	for absolutePath, parsed := range parsedByPath {
		if !selectedSet[absolutePath] {
			continue
		}
		source, readErr := os.ReadFile(absolutePath)
		if readErr != nil {
			return readErr
		}
		relative, relativeErr := workspaceRelative(absoluteRoot, absolutePath)
		if relativeErr != nil {
			return relativeErr
		}
		for _, declaration := range parsed.Decls {
			function, ok := declaration.(*ast.FuncDecl)
			if !ok || function.Body == nil {
				continue
			}
			object, ok := info.Defs[function.Name].(*types.Func)
			if !ok {
				return fmt.Errorf("%s: Go type account omitted function %s", relative, function.Name.Name)
			}
			signature, ok := object.Type().(*types.Signature)
			if !ok {
				return fmt.Errorf("%s: Go type account omitted signature %s", relative, function.Name.Name)
			}
			start := set.Position(function.Pos()).Offset
			end := set.Position(function.End()).Offset
			fingerprint := sha256.Sum256(source[start:end])
			encodedFingerprint := "sha256:" + hex.EncodeToString(fingerprint[:])
			site := function.Name.Name
			semanticSite := goSemanticSite(packagePath, function.Name.Name, signature)
			ast.Inspect(function.Body, func(node ast.Node) bool {
				call, ok := node.(*ast.CallExpr)
				if !ok {
					return true
				}
				name := resolvedMarkerName(call.Fun, info)
				if name == "Covers" || name == "CoversMechanism" {
					err = fmt.Errorf(
						"%s:%d: retired alpha 1 marker %s is not supported",
						relative,
						set.Position(call.Pos()).Line,
						name,
					)
					return false
				}
				if name != "Realizes" && name != "ImplementsCheck" && name != "ImplementsMechanism" {
					return true
				}
				if insideFunctionLiteral(function.Body, call.Pos()) {
					err = fmt.Errorf(
						"%s:%d: marker inside an anonymous function has no stable Go site",
						relative,
						set.Position(call.Pos()).Line,
					)
					return false
				}
				values, valueErr := stringArguments(call.Args)
				if valueErr != nil {
					err = fmt.Errorf("%s:%d: %s", relative, set.Position(call.Pos()).Line, valueErr)
					return false
				}
				valueErr = appendMarker(
					result,
					name,
					values,
					site,
					semanticSite,
					relative,
					encodedFingerprint,
					mechanismSites,
				)
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
	}
	return nil
}

func resolvedMarkerName(expression ast.Expr, info *types.Info) string {
	var object types.Object
	switch value := expression.(type) {
	case *ast.Ident:
		object = info.Uses[value]
	case *ast.SelectorExpr:
		object = info.Uses[value.Sel]
	default:
		return ""
	}
	function, ok := object.(*types.Func)
	if !ok || function.Pkg() == nil || function.Pkg().Path() != annotationPackage {
		return ""
	}
	return function.Name()
}

func preflightMarkerCalls(
	files map[string]*ast.File,
	selected map[string]bool,
	set *token.FileSet,
) error {
	for path, parsed := range files {
		if !selected[path] {
			continue
		}
		aliases, dotImport := markerImports(parsed)
		var finding error
		ast.Inspect(parsed, func(node ast.Node) bool {
			if finding != nil {
				return false
			}
			call, ok := node.(*ast.CallExpr)
			if !ok {
				return true
			}
			name := callName(call.Fun, aliases, dotImport)
			if name == "Covers" || name == "CoversMechanism" {
				finding = fmt.Errorf(
					"%s:%d: retired alpha 1 marker %s is not supported",
					path,
					set.Position(call.Pos()).Line,
					name,
				)
				return false
			}
			required := map[string]int{
				"Realizes": 2, "ImplementsCheck": 1, "ImplementsMechanism": 2,
			}
			count, marker := required[name]
			if !marker {
				return true
			}
			values, err := stringArguments(call.Args)
			if err != nil {
				finding = fmt.Errorf("%s:%d: %s", path, set.Position(call.Pos()).Line, err)
				return false
			}
			if len(values) != count {
				finding = fmt.Errorf("%s needs exactly %d arguments", name, count)
				return false
			}
			return true
		})
		if finding != nil {
			return finding
		}
	}
	return nil
}

func insideFunctionLiteral(body *ast.BlockStmt, position token.Pos) bool {
	inside := false
	ast.Inspect(body, func(node ast.Node) bool {
		literal, ok := node.(*ast.FuncLit)
		if ok && literal.Pos() <= position && position <= literal.End() {
			inside = true
			return false
		}
		return !inside
	})
	return inside
}

func markerImports(file *ast.File) (map[string]bool, bool) {
	aliases := map[string]bool{}
	dotImport := false
	for _, imported := range file.Imports {
		path, err := strconv.Unquote(imported.Path.Value)
		if err != nil || path != annotationPackage {
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

type boundedBuffer struct {
	content  bytes.Buffer
	limit    int
	overflow bool
}

func (buffer *boundedBuffer) Write(content []byte) (int, error) {
	remaining := buffer.limit - buffer.content.Len()
	if remaining > 0 {
		retained := content
		if len(retained) > remaining {
			retained = retained[:remaining]
		}
		_, _ = buffer.content.Write(retained)
	}
	if len(content) > remaining {
		buffer.overflow = true
	}
	return len(content), nil
}

type listedPackage struct {
	ImportPath string `json:"ImportPath"`
	Export     string `json:"Export"`
}

func packageImporter(directory string, set *token.FileSet) (types.Importer, error) {
	command := exec.Command("go", "list", "-deps", "-test", "-export", "-json", ".")
	command.Dir = directory
	stdout := &boundedBuffer{limit: 64 * 1024 * 1024}
	stderr := &boundedBuffer{limit: 1024 * 1024}
	command.Stdout = stdout
	command.Stderr = stderr
	if err := command.Run(); err != nil {
		return nil, fmt.Errorf(
			"%s: Go compiler account failed: %s",
			directory,
			strings.TrimSpace(stderr.content.String()),
		)
	}
	if stdout.overflow || stderr.overflow {
		return nil, fmt.Errorf("%s: Go compiler account exceeded its output bound", directory)
	}
	exports := map[string]string{}
	decoder := json.NewDecoder(bytes.NewReader(stdout.content.Bytes()))
	for {
		var listed listedPackage
		if err := decoder.Decode(&listed); errors.Is(err, io.EOF) {
			break
		} else if err != nil {
			return nil, fmt.Errorf("%s: malformed go list output: %w", directory, err)
		}
		if listed.ImportPath == "" || listed.Export == "" {
			continue
		}
		canonical, err := canonicalPath(listed.Export)
		if err != nil {
			return nil, fmt.Errorf(
				"%s: cannot resolve export data for %s: %w",
				directory,
				listed.ImportPath,
				err,
			)
		}
		if prior, present := exports[listed.ImportPath]; present && prior != canonical {
			return nil, fmt.Errorf(
				"%s: Go compiler reported conflicting export data for %s",
				directory,
				listed.ImportPath,
			)
		}
		exports[listed.ImportPath] = canonical
	}
	lookup := func(path string) (io.ReadCloser, error) {
		export, present := exports[path]
		if !present {
			return nil, fmt.Errorf("Go compiler account omitted export data for %s", path)
		}
		return os.Open(export)
	}
	return importer.ForCompiler(set, "gc", lookup), nil
}

func validateMechanismSites(implementations []mechanismImplementation) error {
	sites := map[string]mechanismImplementation{}
	for _, implementation := range implementations {
		if prior, present := sites[implementation.Site]; present {
			return fmt.Errorf(
				"%s: ambiguous mechanism site %q for %s#%s in %s and %s#%s",
				implementation.File,
				implementation.Site,
				prior.Spec,
				prior.Mechanism,
				prior.File,
				implementation.Spec,
				implementation.Mechanism,
			)
		}
		sites[implementation.Site] = implementation
	}
	return nil
}

func packageImportPath(directory string) (string, error) {
	cursor := directory
	for {
		manifestPath := filepath.Join(cursor, "go.mod")
		content, err := os.ReadFile(manifestPath)
		if err == nil {
			var module string
			for _, line := range strings.Split(string(content), "\n") {
				fields := strings.Fields(line)
				if len(fields) == 2 && fields[0] == "module" {
					module = fields[1]
					break
				}
			}
			if module == "" {
				return "", fmt.Errorf("%s: go.mod has no module identity", manifestPath)
			}
			relative, relativeErr := filepath.Rel(cursor, directory)
			if relativeErr != nil {
				return "", relativeErr
			}
			if relative == "." {
				return module, nil
			}
			return strings.TrimSuffix(module, "/") + "/" + filepath.ToSlash(relative), nil
		}
		parent := filepath.Dir(cursor)
		if parent == cursor {
			return "", fmt.Errorf("%s: cannot derive Go package import identity", directory)
		}
		cursor = parent
	}
}

func goSemanticSite(packagePath string, name string, signature *types.Signature) string {
	qualifier := func(other *types.Package) string {
		if other.Path() == packagePath {
			return ""
		}
		return other.Path()
	}
	receiver := ""
	positions := positionalTypeParameters(signature)
	if signature.Recv() != nil {
		receiver = "(" + canonicalGoType(signature.Recv().Type(), qualifier, positions) + ")."
	}
	return packagePath + "." + receiver + name + canonicalSignature(signature, qualifier, positions)
}

func positionalTypeParameters(signature *types.Signature) map[string]string {
	positions := map[string]string{}
	index := 0
	add := func(parameters *types.TypeParamList) {
		if parameters == nil {
			return
		}
		for offset := 0; offset < parameters.Len(); offset++ {
			positions[parameters.At(offset).Obj().Name()] = fmt.Sprintf("$%d", index)
			index++
		}
	}
	add(signature.RecvTypeParams())
	add(signature.TypeParams())
	return positions
}

func canonicalGoType(value types.Type, qualifier types.Qualifier, positions map[string]string) string {
	return replaceTypeParameters(types.TypeString(value, qualifier), positions)
}

func replaceTypeParameters(value string, positions map[string]string) string {
	runes := []rune(value)
	var result strings.Builder
	for index := 0; index < len(runes); {
		if runes[index] != '_' && !unicode.IsLetter(runes[index]) {
			result.WriteRune(runes[index])
			index++
			continue
		}
		end := index + 1
		for end < len(runes) && (runes[end] == '_' || unicode.IsLetter(runes[end]) || unicode.IsDigit(runes[end])) {
			end++
		}
		name := string(runes[index:end])
		previousDot := index > 0 && runes[index-1] == '.'
		nextDot := end < len(runes) && runes[end] == '.'
		if position, present := positions[name]; present && !previousDot && !nextDot {
			result.WriteString(position)
		} else {
			result.WriteString(name)
		}
		index = end
	}
	return result.String()
}

func canonicalSignature(signature *types.Signature, qualifier types.Qualifier, positions map[string]string) string {
	parameters := make([]string, signature.Params().Len())
	for index := range parameters {
		parameterType := signature.Params().At(index).Type()
		if signature.Variadic() && index == len(parameters)-1 {
			if slice, ok := parameterType.(*types.Slice); ok {
				parameters[index] = "..." + canonicalGoType(slice.Elem(), qualifier, positions)
				continue
			}
		}
		parameters[index] = canonicalGoType(parameterType, qualifier, positions)
	}
	results := make([]string, signature.Results().Len())
	for index := range results {
		results[index] = canonicalGoType(signature.Results().At(index).Type(), qualifier, positions)
	}
	typeParameters := ""
	allParameters := []*types.TypeParam{}
	for _, list := range []*types.TypeParamList{signature.RecvTypeParams(), signature.TypeParams()} {
		if list != nil {
			for index := 0; index < list.Len(); index++ {
				allParameters = append(allParameters, list.At(index))
			}
		}
	}
	if len(allParameters) > 0 {
		constraints := make([]string, len(allParameters))
		for index, parameter := range allParameters {
			constraints[index] = fmt.Sprintf("$%d:%s", index, canonicalGoType(parameter.Constraint(), qualifier, positions))
		}
		typeParameters = "[" + strings.Join(constraints, ",") + "]"
	}
	return typeParameters + "(" + strings.Join(parameters, ",") + ")->(" +
		strings.Join(results, ",") + ")"
}

func sortManifest(result *manifest) {
	sort.Slice(result.Realizes, func(left, right int) bool {
		return fmt.Sprint(result.Realizes[left]) < fmt.Sprint(result.Realizes[right])
	})
	sort.Slice(result.CheckImplementations, func(left, right int) bool {
		return fmt.Sprint(result.CheckImplementations[left]) <
			fmt.Sprint(result.CheckImplementations[right])
	})
	sort.Slice(result.MechanismImplementations, func(left, right int) bool {
		return fmt.Sprint(result.MechanismImplementations[left]) <
			fmt.Sprint(result.MechanismImplementations[right])
	})
	sort.Slice(result.Artifacts, func(left, right int) bool {
		return fmt.Sprint(result.Artifacts[left]) < fmt.Sprint(result.Artifacts[right])
	})
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

func appendMarker(
	result *manifest,
	name string,
	values []string,
	site string,
	semanticSite string,
	file string,
	fingerprint string,
	mechanismSites map[string][2]string,
) error {
	requiredArguments := map[string]int{"Realizes": 2, "ImplementsCheck": 1, "ImplementsMechanism": 2}
	required, marker := requiredArguments[name]
	if !marker {
		return nil
	}
	if len(values) != required {
		return fmt.Errorf("%s needs exactly %d arguments", name, required)
	}
	switch name {
	case "Realizes":
		result.Realizes = append(result.Realizes, relation{Spec: values[0], Scenario: values[1], Site: site, File: file, Lang: "go", SourceFingerprint: fingerprint})
	case "ImplementsCheck":
		result.CheckImplementations = append(result.CheckImplementations, checkImplementation{Check: values[0], Site: site, File: file, Lang: "go", SourceFingerprint: fingerprint})
	case "ImplementsMechanism":
		if strings.Contains(semanticSite, "invalid type") {
			return fmt.Errorf("Go type account could not resolve mechanism signature %s", site)
		}
		target := [2]string{values[0], values[1]}
		if prior, present := mechanismSites[semanticSite]; present {
			return fmt.Errorf(
				"ambiguous mechanism site %q for %s#%s and %s#%s",
				semanticSite,
				prior[0],
				prior[1],
				target[0],
				target[1],
			)
		}
		mechanismSites[semanticSite] = target
		binding := "go-symbol:" + semanticSite
		result.MechanismImplementations = append(result.MechanismImplementations, mechanismImplementation{Spec: values[0], Mechanism: values[1], Site: semanticSite, Binding: binding, File: file, Lang: "go", SourceFingerprint: fingerprint})
		result.Artifacts = append(result.Artifacts, artifact{ID: binding, Kind: "go-symbol", File: file})
	}
	return nil
}
