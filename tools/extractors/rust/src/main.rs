use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use proc_macro2::{LineColumn, Span};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, FnArg, GenericParam, ImplItemFn, Item, ItemFn, LitStr, Signature, Token, TraitItemFn,
};

#[derive(Debug, Clone, Eq, PartialEq)]
struct Marker {
    kind: String,
    values: Vec<String>,
    site: String,
    file: String,
    fingerprint: String,
}

#[derive(Debug, Clone)]
struct SemanticContext {
    target_kind: String,
    target_name: String,
    crate_name: String,
    module: Vec<String>,
    manifest: PathBuf,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct CargoTarget {
    kind: String,
    name: String,
    crate_name: String,
    source: PathBuf,
    manifest: PathBuf,
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("azimuth-emit-rust: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut output = None;
    let mut root = PathBuf::from(".");
    let mut inputs = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--output" | "-o" => {
                output = Some(PathBuf::from(value(&args, index, "--output")?));
                index += 2;
            }
            "--root" => {
                root = PathBuf::from(value(&args, index, "--root")?);
                index += 2;
            }
            option if option.starts_with('-') => return Err(format!("unknown option `{option}`")),
            input => {
                inputs.push(PathBuf::from(input));
                index += 1;
            }
        }
    }
    let output =
        output.ok_or("usage: azimuth-emit-rust --output <path> [--root <dir>] <input>...")?;
    if inputs.is_empty() {
        return Err("at least one input is required".into());
    }
    let markers = emit(&inputs, &root)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(&output, manifest_json(&markers))
        .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    Ok(())
}

fn value(args: &[String], index: usize, name: &str) -> Result<String, String> {
    args.get(index + 1)
        .cloned()
        .ok_or_else(|| format!("`{name}` needs a value"))
}

fn emit(inputs: &[PathBuf], root: &Path) -> Result<Vec<Marker>, String> {
    let semantic_root = root
        .canonicalize()
        .map_err(|error| format!("{}: cannot resolve --root: {error}", root.display()))?;
    if !semantic_root.is_dir() {
        return Err(format!("{}: --root must be a directory", root.display()));
    }
    let mut files = Vec::new();
    for input in inputs {
        let selected = input
            .canonicalize()
            .map_err(|error| format!("{}: cannot resolve input: {error}", input.display()))?;
        if selected != semantic_root {
            normalized_relative(&semantic_root, &selected)?;
        }
        collect(&selected, &semantic_root, &mut files)?;
    }
    files.sort();
    files.dedup();
    let mut markers = Vec::new();
    let mut checked = BTreeSet::new();
    for file in files {
        let relative = normalized_relative(&semantic_root, &file)?;
        let source = fs::read_to_string(&file).map_err(|error| error.to_string())?;
        let context = semantic_context(&file)?;
        let target = CargoTarget {
            kind: context.target_kind.clone(),
            name: context.target_name.clone(),
            crate_name: context.crate_name.clone(),
            source: PathBuf::new(),
            manifest: context.manifest.clone(),
        };
        if checked.insert(target.clone()) {
            check_target(&target)?;
        }
        let file_markers = scan(&source, &relative, &context)?;
        for marker in file_markers {
            markers.push(marker);
        }
    }
    validate_mechanism_sites(&markers)?;
    markers.sort_by(|left, right| {
        (&left.kind, &left.values, &left.site, &left.file).cmp(&(
            &right.kind,
            &right.values,
            &right.site,
            &right.file,
        ))
    });
    Ok(markers)
}

fn validate_mechanism_sites(markers: &[Marker]) -> Result<(), String> {
    let mut mechanism_sites = BTreeMap::<String, (String, String)>::new();
    for marker in markers
        .iter()
        .filter(|marker| marker.kind == "implements_mechanism")
    {
        let target = (marker.values[0].clone(), marker.values[1].clone());
        if let Some(prior) = mechanism_sites.insert(marker.site.clone(), target.clone()) {
            return Err(format!(
                "{}: ambiguous mechanism site `{}` for {}#{} and {}#{}",
                marker.file, marker.site, prior.0, prior.1, target.0, target.1
            ));
        }
    }
    Ok(())
}

fn normalized_relative(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("{}: input is outside --root", path.display()))?;
    let parts = relative
        .components()
        .map(|part| part.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| format!("{}: file is not a UTF-8 workspace path", path.display()))?;
    if parts.is_empty()
        || parts
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == ".." || part.contains('\\'))
    {
        return Err(format!(
            "{}: file is not a normalized workspace-relative path",
            path.display()
        ));
    }
    Ok(parts.join("/"))
}

fn collect(path: &Path, root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            let canonical = path.canonicalize().map_err(|error| error.to_string())?;
            normalized_relative(root, &canonical)?;
            files.push(canonical);
        } else {
            return Err(format!(
                "{}: Rust input must be an .rs file or directory",
                path.display()
            ));
        }
        return Ok(());
    }
    for entry in
        fs::read_dir(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_name() == "target" || entry.file_name() == ".git" {
            continue;
        }
        collect(&entry.path(), root, files)?;
    }
    Ok(())
}

fn semantic_context(file: &Path) -> Result<SemanticContext, String> {
    let absolute = file.canonicalize().map_err(|error| error.to_string())?;
    let mut directory = absolute
        .parent()
        .ok_or_else(|| format!("{}: source has no parent", file.display()))?;
    let manifest = loop {
        let candidate = directory.join("Cargo.toml");
        if candidate.is_file() {
            break candidate;
        }
        directory = directory
            .parent()
            .ok_or_else(|| format!("{}: cannot derive a Cargo crate identity", file.display()))?;
    };
    let targets = cargo_targets(&manifest)?;
    let mut reached = Vec::new();
    for target in targets {
        let modules = reachable_modules(&target)?;
        if let Some(module) = modules.get(&absolute) {
            reached.push((target, module.clone()));
        }
    }
    match reached.as_slice() {
        [] => Err(format!(
            "{}: Rust source is unreachable from a conventional Cargo target",
            file.display()
        )),
        [(target, module)] => Ok(SemanticContext {
            target_kind: target.kind.clone(),
            target_name: target.name.clone(),
            crate_name: target.crate_name.clone(),
            module: module.clone(),
            manifest: target.manifest.clone(),
        }),
        _ => Err(format!(
            "{}: Rust source is reachable from several Cargo targets",
            file.display()
        )),
    }
}

fn cargo_targets(manifest: &Path) -> Result<Vec<CargoTarget>, String> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(manifest)
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .map_err(|error| format!("cannot execute cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{}: cargo metadata failed: {}",
            manifest.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("{}: invalid cargo metadata: {error}", manifest.display()))?;
    let canonical_manifest = manifest.canonicalize().map_err(|error| error.to_string())?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| "cargo metadata omitted packages".to_string())?;
    let package = packages
        .iter()
        .find(|package| {
            package["manifest_path"]
                .as_str()
                .and_then(|path| Path::new(path).canonicalize().ok())
                .is_some_and(|path| path == canonical_manifest)
        })
        .ok_or_else(|| {
            format!(
                "{}: cargo metadata omitted owning package",
                manifest.display()
            )
        })?;
    let package_name = package["name"]
        .as_str()
        .ok_or_else(|| format!("{}: package has no name", manifest.display()))?;
    let package_root = canonical_manifest
        .parent()
        .ok_or_else(|| format!("{}: manifest has no parent", manifest.display()))?;
    let target_values = package["targets"]
        .as_array()
        .ok_or_else(|| format!("{}: package has no targets", manifest.display()))?;
    let mut targets = Vec::new();
    for value in target_values {
        let name = value["name"]
            .as_str()
            .ok_or_else(|| "cargo target has no name".to_string())?;
        let kinds = value["kind"]
            .as_array()
            .ok_or_else(|| "cargo target has no kind".to_string())?;
        if kinds
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|kind| kind == "custom-build")
        {
            continue;
        }
        let kind = target_kind(kinds)?;
        let source = Path::new(
            value["src_path"]
                .as_str()
                .ok_or_else(|| "cargo target has no source path".to_string())?,
        )
        .canonicalize()
        .map_err(|error| error.to_string())?;
        if !conventional_target_source(package_root, package_name, &kind, name, &source) {
            return Err(format!(
                "{}: Cargo target `{name}` uses unsupported custom source {}",
                manifest.display(),
                source.display()
            ));
        }
        targets.push(CargoTarget {
            kind,
            name: name.to_string(),
            crate_name: name.replace('-', "_"),
            source,
            manifest: canonical_manifest.clone(),
        });
    }
    targets.sort();
    Ok(targets)
}

fn target_kind(values: &[serde_json::Value]) -> Result<String, String> {
    let names = values
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    for kind in ["proc-macro", "lib", "bin", "test", "example", "bench"] {
        if names.contains(&kind) {
            return Ok(kind.to_string());
        }
    }
    Err(format!("unsupported Cargo target kind {names:?}"))
}

fn conventional_target_source(
    root: &Path,
    package_name: &str,
    kind: &str,
    name: &str,
    source: &Path,
) -> bool {
    let candidates = match kind {
        "lib" | "proc-macro" => vec![root.join("src/lib.rs")],
        "bin" if name == package_name => vec![root.join("src/main.rs")],
        "bin" => vec![
            root.join(format!("src/bin/{name}.rs")),
            root.join(format!("src/bin/{name}/main.rs")),
        ],
        "test" => vec![
            root.join(format!("tests/{name}.rs")),
            root.join(format!("tests/{name}/main.rs")),
        ],
        "example" => vec![
            root.join(format!("examples/{name}.rs")),
            root.join(format!("examples/{name}/main.rs")),
        ],
        "bench" => vec![
            root.join(format!("benches/{name}.rs")),
            root.join(format!("benches/{name}/main.rs")),
        ],
        _ => Vec::new(),
    };
    candidates.into_iter().any(|candidate| {
        candidate
            .canonicalize()
            .is_ok_and(|candidate| candidate == source)
    })
}

fn reachable_modules(target: &CargoTarget) -> Result<BTreeMap<PathBuf, Vec<String>>, String> {
    let mut modules = BTreeMap::new();
    let mut visiting = BTreeSet::new();
    visit_module_file(&target.source, Vec::new(), &mut modules, &mut visiting)?;
    Ok(modules)
}

fn visit_module_file(
    file: &Path,
    module: Vec<String>,
    modules: &mut BTreeMap<PathBuf, Vec<String>>,
    visiting: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    let canonical = file.canonicalize().map_err(|error| error.to_string())?;
    if let Some(previous) = modules.get(&canonical) {
        if previous != &module {
            return Err(format!(
                "{}: source has ambiguous Rust module identities",
                canonical.display()
            ));
        }
        return Ok(());
    }
    if !visiting.insert(canonical.clone()) {
        return Err(format!("{}: cyclic Rust module graph", canonical.display()));
    }
    let source = fs::read_to_string(&canonical).map_err(|error| error.to_string())?;
    let syntax =
        syn::parse_file(&source).map_err(|error| format!("{}: {error}", canonical.display()))?;
    reject_includes(&syntax, &canonical)?;
    modules.insert(canonical.clone(), module.clone());
    let base = module_child_directory(&canonical);
    visit_module_items(&syntax.items, &base, &module, modules, visiting)?;
    visiting.remove(&canonical);
    Ok(())
}

fn module_child_directory(file: &Path) -> PathBuf {
    let parent = file.parent().unwrap_or_else(|| Path::new("."));
    match file.file_name().and_then(|name| name.to_str()) {
        Some("lib.rs" | "main.rs" | "mod.rs") => parent.to_path_buf(),
        _ => parent.join(file.file_stem().unwrap_or_default()),
    }
}

fn visit_module_items(
    items: &[Item],
    base: &Path,
    module: &[String],
    modules: &mut BTreeMap<PathBuf, Vec<String>>,
    visiting: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    for item in items {
        let Item::Mod(item_mod) = item else { continue };
        if item_mod
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("path"))
        {
            return Err(format!(
                "module `{}` uses unsupported #[path] routing",
                item_mod.ident
            ));
        }
        if has_conditional_attributes(&item_mod.attrs) {
            if item_mod.content.is_none() {
                return Err(format!(
                    "module `{}` uses unsupported conditional source routing",
                    item_mod.ident
                ));
            }
            continue;
        }
        let mut child_module = module.to_vec();
        child_module.push(item_mod.ident.to_string());
        if let Some((_, nested)) = &item_mod.content {
            visit_module_items(
                nested,
                &base.join(item_mod.ident.to_string()),
                &child_module,
                modules,
                visiting,
            )?;
            continue;
        }
        let flat = base.join(format!("{}.rs", item_mod.ident));
        let directory = base.join(item_mod.ident.to_string()).join("mod.rs");
        let flat_exists = flat.is_file();
        let directory_exists = directory.is_file();
        let child = match (flat_exists, directory_exists) {
            (true, false) => flat,
            (false, true) => directory,
            (true, true) => {
                return Err(format!(
                    "module `{}` has ambiguous conventional source files",
                    item_mod.ident
                ))
            }
            (false, false) => {
                return Err(format!(
                    "module `{}` has no conventional source file",
                    item_mod.ident
                ))
            }
        };
        visit_module_file(&child, child_module, modules, visiting)?;
    }
    Ok(())
}

fn reject_includes(syntax: &syn::File, file: &Path) -> Result<(), String> {
    struct IncludeVisitor {
        found: bool,
    }
    impl<'ast> Visit<'ast> for IncludeVisitor {
        fn visit_macro(&mut self, node: &'ast syn::Macro) {
            if node.path.is_ident("include") {
                self.found = true;
            }
            visit::visit_macro(self, node);
        }
    }
    let mut visitor = IncludeVisitor { found: false };
    visitor.visit_file(syntax);
    if visitor.found {
        Err(format!(
            "{}: include! source routing is unsupported",
            file.display()
        ))
    } else {
        Ok(())
    }
}

fn check_target(target: &CargoTarget) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.args(["check", "--quiet", "--offline", "--manifest-path"]);
    command.arg(&target.manifest);
    match target.kind.as_str() {
        "lib" | "proc-macro" => {
            command.arg("--lib");
        }
        "bin" => {
            command.args(["--bin", &target.name]);
        }
        "test" => {
            command.args(["--test", &target.name]);
        }
        "example" => {
            command.args(["--example", &target.name]);
        }
        "bench" => {
            command.args(["--bench", &target.name]);
        }
        _ => return Err(format!("unsupported Cargo target kind `{}`", target.kind)),
    }
    let output = command
        .output()
        .map_err(|error| format!("cannot execute cargo check: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} target `{}` did not compile: {}",
            target.kind,
            target.name,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn scan(source: &str, file: &str, context: &SemanticContext) -> Result<Vec<Marker>, String> {
    let syntax = syn::parse_file(source).map_err(|error| format!("{file}: {error}"))?;
    let mut visitor = MarkerVisitor {
        source,
        file,
        context,
        declaration: Vec::new(),
        generic_scopes: Vec::new(),
        conditional_depth: 0,
        markers: Vec::new(),
        error: None,
    };
    visitor.visit_file(&syntax);
    if let Some(error) = visitor.error {
        Err(error)
    } else {
        Ok(visitor.markers)
    }
}

struct MarkerVisitor<'a> {
    source: &'a str,
    file: &'a str,
    context: &'a SemanticContext,
    declaration: Vec<String>,
    generic_scopes: Vec<BTreeMap<String, String>>,
    conditional_depth: usize,
    markers: Vec<Marker>,
    error: Option<String>,
}

impl MarkerVisitor<'_> {
    fn record(&mut self, attributes: &[Attribute], signature: &Signature, span: Span) {
        if self.error.is_some() {
            return;
        }
        let mut found = Vec::new();
        for attribute in attributes {
            match marker_attribute(attribute) {
                Ok(Some(marker)) => found.push((attribute.span(), marker)),
                Ok(None) => {}
                Err(error) => {
                    self.error = Some(format!("{}: {error}", self.file));
                    return;
                }
            }
        }
        if found.is_empty() {
            return;
        }
        if self.conditional_depth > 0
            || attributes.iter().any(|attribute| {
                attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr")
            })
        {
            self.error = Some(format!(
                "{}: marked declaration uses conditional compilation",
                self.file
            ));
            return;
        }
        let mut positions = self.positions();
        extend_generic_positions(&mut positions, &signature.generics.params);
        let signature_text = canonical_signature(signature, &positions);
        let mut path = Vec::new();
        path.extend(self.context.module.iter().cloned());
        path.extend(self.declaration.iter().cloned());
        path.push(signature.ident.to_string());
        let semantic_site = format!(
            "cargo:{}:{}:{}::{} {}",
            self.context.target_kind,
            self.context.target_name,
            self.context.crate_name,
            path.join("::"),
            signature_text
        );
        let start = found
            .iter()
            .map(|(span, _)| span.start())
            .min_by_key(|value| (value.line, value.column))
            .unwrap_or_else(|| span.start());
        let segment = match source_segment(self.source, start, span.end()) {
            Ok(value) => value,
            Err(error) => {
                self.error = Some(format!("{}: {error}", self.file));
                return;
            }
        };
        let fingerprint = stable_fingerprint(segment);
        for (_, (kind, values)) in found {
            let site = if kind == "implements_mechanism" {
                semantic_site.clone()
            } else {
                signature.ident.to_string()
            };
            self.markers.push(Marker {
                kind,
                values,
                site,
                file: self.file.into(),
                fingerprint: fingerprint.clone(),
            });
        }
    }

    fn positions(&self) -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();
        for scope in &self.generic_scopes {
            for (name, position) in scope {
                result.insert(name.clone(), position.clone());
            }
        }
        result
    }
}

impl<'ast> Visit<'ast> for MarkerVisitor<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if let Some((_, items)) = &node.content {
            let conditional = has_conditional_attributes(&node.attrs);
            self.conditional_depth += usize::from(conditional);
            self.declaration.push(node.ident.to_string());
            for item in items {
                self.visit_item(item);
            }
            self.declaration.pop();
            self.conditional_depth -= usize::from(conditional);
        }
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let conditional = has_conditional_attributes(&node.attrs);
        self.conditional_depth += usize::from(conditional);
        let mut positions = self.positions();
        let scope = extend_generic_positions(&mut positions, &node.generics.params);
        let self_type =
            replace_generic_tokens(&node.self_ty.to_token_stream().to_string(), &positions);
        let identity = if let Some((_, trait_path, _)) = &node.trait_ {
            format!(
                "<{} as {}>",
                self_type,
                replace_generic_tokens(&trait_path.to_token_stream().to_string(), &positions)
            )
        } else {
            self_type
        };
        let constraints = canonical_generics(&node.generics, &positions);
        self.declaration.push(format!("{identity}{constraints}"));
        self.generic_scopes.push(scope);
        visit::visit_item_impl(self, node);
        self.generic_scopes.pop();
        self.declaration.pop();
        self.conditional_depth -= usize::from(conditional);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let conditional = has_conditional_attributes(&node.attrs);
        self.conditional_depth += usize::from(conditional);
        let mut positions = self.positions();
        let scope = extend_generic_positions(&mut positions, &node.generics.params);
        self.declaration.push(format!(
            "{}{}",
            node.ident,
            canonical_generics(&node.generics, &positions)
        ));
        self.generic_scopes.push(scope);
        visit::visit_item_trait(self, node);
        self.generic_scopes.pop();
        self.declaration.pop();
        self.conditional_depth -= usize::from(conditional);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.record(&node.attrs, &node.sig, node.span());
        self.declaration.push(node.sig.ident.to_string());
        visit::visit_block(self, &node.block);
        self.declaration.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.record(&node.attrs, &node.sig, node.span());
        self.declaration.push(node.sig.ident.to_string());
        visit::visit_block(self, &node.block);
        self.declaration.pop();
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        self.record(&node.attrs, &node.sig, node.span());
        if let Some(block) = &node.default {
            self.declaration.push(node.sig.ident.to_string());
            visit::visit_block(self, block);
            self.declaration.pop();
        }
    }
}

fn has_conditional_attributes(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
}

fn extend_generic_positions(
    positions: &mut BTreeMap<String, String>,
    parameters: &Punctuated<GenericParam, Token![,]>,
) -> BTreeMap<String, String> {
    let mut scope = BTreeMap::new();
    let mut index = positions.len();
    for parameter in parameters {
        let name = match parameter {
            GenericParam::Lifetime(value) => format!("'{}", value.lifetime.ident),
            GenericParam::Type(value) => value.ident.to_string(),
            GenericParam::Const(value) => value.ident.to_string(),
        };
        let position = format!("${index}");
        positions.insert(name.clone(), position.clone());
        scope.insert(name, position);
        index += 1;
    }
    scope
}

fn canonical_generics(generics: &syn::Generics, positions: &BTreeMap<String, String>) -> String {
    let parameters =
        replace_generic_tokens(&generics.params.to_token_stream().to_string(), positions);
    let where_clause = generics
        .where_clause
        .as_ref()
        .map(|clause| replace_generic_tokens(&clause.to_token_stream().to_string(), positions))
        .unwrap_or_default();
    if parameters.is_empty() && where_clause.is_empty() {
        String::new()
    } else if where_clause.is_empty() {
        format!("<{parameters}>")
    } else if parameters.is_empty() {
        format!(" {where_clause}")
    } else {
        format!("<{parameters}> {where_clause}")
    }
}

fn canonical_signature(signature: &Signature, positions: &BTreeMap<String, String>) -> String {
    let mut qualifiers = Vec::new();
    if signature.constness.is_some() {
        qualifiers.push("const".to_string());
    }
    if signature.asyncness.is_some() {
        qualifiers.push("async".to_string());
    }
    if signature.unsafety.is_some() {
        qualifiers.push("unsafe".to_string());
    }
    if let Some(abi) = &signature.abi {
        qualifiers.push(replace_generic_tokens(
            &abi.to_token_stream().to_string(),
            positions,
        ));
    }
    qualifiers.push("fn".to_string());
    let generic_parameters = replace_generic_tokens(
        &signature.generics.params.to_token_stream().to_string(),
        positions,
    );
    let generics = if generic_parameters.is_empty() {
        String::new()
    } else {
        format!("<{generic_parameters}>")
    };
    let inputs = signature
        .inputs
        .iter()
        .map(|input| match input {
            FnArg::Receiver(receiver) => {
                replace_generic_tokens(&receiver.to_token_stream().to_string(), positions)
            }
            FnArg::Typed(parameter) => {
                replace_generic_tokens(&parameter.ty.to_token_stream().to_string(), positions)
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    let variadic = signature
        .variadic
        .as_ref()
        .map(|value| value.to_token_stream().to_string())
        .unwrap_or_default();
    let arguments = if variadic.is_empty() {
        inputs
    } else if inputs.is_empty() {
        variadic
    } else {
        format!("{inputs},{variadic}")
    };
    let output = replace_generic_tokens(&signature.output.to_token_stream().to_string(), positions);
    let where_clause = signature
        .generics
        .where_clause
        .as_ref()
        .map(|clause| replace_generic_tokens(&clause.to_token_stream().to_string(), positions))
        .unwrap_or_default();
    let suffix = if where_clause.is_empty() {
        String::new()
    } else {
        format!(" {where_clause}")
    };
    format!(
        "{}{}({arguments}){output}{suffix}",
        qualifiers.join(" "),
        generics
    )
}

fn replace_generic_tokens(value: &str, positions: &BTreeMap<String, String>) -> String {
    let characters = value.chars().collect::<Vec<_>>();
    let mut result = String::new();
    let mut index = 0;
    while index < characters.len() {
        if characters[index] == '\'' && index + 1 < characters.len() {
            let mut end = index + 2;
            while end < characters.len()
                && (characters[end] == '_' || characters[end].is_alphanumeric())
            {
                end += 1;
            }
            let name = characters[index..end].iter().collect::<String>();
            if let Some(position) = positions.get(&name) {
                result.push_str(position);
            } else {
                result.push_str(&name);
            }
            index = end;
            continue;
        }
        if characters[index] == '_' || characters[index].is_alphabetic() {
            let mut end = index + 1;
            while end < characters.len()
                && (characters[end] == '_' || characters[end].is_alphanumeric())
            {
                end += 1;
            }
            let name = characters[index..end].iter().collect::<String>();
            let previous_path = index > 0 && characters[index - 1] == ':';
            if let Some(position) = positions.get(&name).filter(|_| !previous_path) {
                result.push_str(position);
            } else {
                result.push_str(&name);
            }
            index = end;
            continue;
        }
        result.push(characters[index]);
        index += 1;
    }
    result
}

fn marker_attribute(attribute: &Attribute) -> Result<Option<(String, Vec<String>)>, String> {
    let segments = attribute
        .path()
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let Some(name) = segments.last().map(String::as_str) else {
        return Ok(None);
    };
    let accepted_prefix = segments.len() == 1
        || (segments.len() == 2
            && matches!(segments[0].as_str(), "azimuth" | "azimuth_annotations"));
    if !accepted_prefix {
        return Ok(None);
    }
    if matches!(name, "covers" | "covers_mechanism") {
        return Err(format!("retired alpha 1 marker {name} is not supported"));
    }
    if !matches!(
        name,
        "realizes" | "implements_check" | "implements_mechanism"
    ) {
        return Ok(None);
    }
    let parser = Punctuated::<LitStr, Token![,]>::parse_terminated;
    let arguments = attribute
        .parse_args_with(parser)
        .map_err(|_| "marker arguments must be string literals".to_string())?;
    let values = arguments
        .iter()
        .map(|value| value.value())
        .collect::<Vec<_>>();
    let required = if name == "implements_check" { 1 } else { 2 };
    if values.len() != required {
        return Err(format!("{name} needs exactly {required} arguments"));
    }
    Ok(Some((name.into(), values)))
}

fn source_segment(source: &str, start: LineColumn, end: LineColumn) -> Result<&str, String> {
    let offset = |position: LineColumn| -> Option<usize> {
        let line_start = source
            .split_inclusive('\n')
            .take(position.line.saturating_sub(1))
            .map(str::len)
            .sum::<usize>();
        let result = line_start.checked_add(position.column)?;
        (result <= source.len()).then_some(result)
    };
    let begin = offset(start).ok_or("marker source span is invalid")?;
    let finish = offset(end).ok_or("declaration source span is invalid")?;
    source
        .get(begin..finish)
        .ok_or_else(|| "declaration source span is not UTF-8 aligned".into())
}

fn stable_fingerprint(source: &str) -> String {
    format!("sha256:{}", sha256(source.as_bytes()))
}

fn sha256(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (input.len() as u64) * 8;
    let mut message = input.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = INITIAL;
    for chunk in message.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, bytes) in chunk.chunks_exact(4).enumerate() {
            words[index] = u32::from_be_bytes(bytes.try_into().unwrap());
        }
        for index in 16..64 {
            let first = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let second = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(first)
                .wrapping_add(words[index - 7])
                .wrapping_add(second);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let first = h
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let second = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(first);
            d = c;
            c = b;
            b = a;
            a = first.wrapping_add(second);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

fn manifest_json(markers: &[Marker]) -> String {
    let realizes = markers
        .iter()
        .filter(|item| item.kind == "realizes")
        .map(relation_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let checks = markers
        .iter()
        .filter(|item| item.kind == "implements_check")
        .map(check_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let implementations = markers
        .iter()
        .filter(|item| item.kind == "implements_mechanism")
        .map(implementation_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let artifacts = markers
        .iter()
        .filter(|item| item.kind == "implements_mechanism")
        .map(artifact_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    format!("{{\n  \"realizes\": [{}],\n  \"check_implementations\": [{}],\n  \"mechanism_implementations\": [{}],\n  \"class_members\": [],\n  \"enumerations\": [],\n  \"artifacts\": [{}]\n}}\n", array_body(&realizes), array_body(&checks), array_body(&implementations), array_body(&artifacts))
}

fn check_json(marker: &Marker) -> String {
    object(&[
        ("check", &marker.values[0]),
        ("site", &marker.site),
        ("file", &marker.file),
        ("lang", "rust"),
        ("source_fingerprint", &marker.fingerprint),
    ])
}

fn implementation_json(marker: &Marker) -> String {
    let binding = format!("rust-symbol:{}", marker.site);
    object(&[
        ("spec", &marker.values[0]),
        ("mechanism", &marker.values[1]),
        ("site", &marker.site),
        ("binding", &binding),
        ("file", &marker.file),
        ("lang", "rust"),
        ("source_fingerprint", &marker.fingerprint),
    ])
}

fn artifact_json(marker: &Marker) -> String {
    let binding = format!("rust-symbol:{}", marker.site);
    object(&[
        ("id", &binding),
        ("kind", "rust-symbol"),
        ("file", &marker.file),
    ])
}

fn array_body(values: &str) -> String {
    if values.is_empty() {
        String::new()
    } else {
        format!("\n    {values}\n  ")
    }
}

fn relation_json(marker: &Marker) -> String {
    let fields = vec![
        ("spec", marker.values[0].as_str()),
        ("scenario", marker.values[1].as_str()),
        ("site", marker.site.as_str()),
        ("file", marker.file.as_str()),
        ("lang", "rust"),
        ("source_fingerprint", marker.fingerprint.as_str()),
    ];
    object(&fields)
}

fn object(fields: &[(&str, &str)]) -> String {
    format!(
        "{{{}}}",
        fields
            .iter()
            .map(|(key, value)| format!("\"{key}\":\"{}\"", escape(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TemporaryProject(PathBuf);

    impl Drop for TemporaryProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn temporary_project(name: &str) -> TemporaryProject {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "azimuth-rust-extractor-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(path.join("src")).unwrap();
        TemporaryProject(path)
    }

    fn annotations_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .unwrap()
            .join("packages/rust/azimuth-annotations")
    }

    fn write_package(project: &Path, extra: &str) {
        fs::write(
            project.join("Cargo.toml"),
            format!(
                "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2021\"\n{extra}\n[dependencies]\nazimuth-annotations = {{ path = {:?} }}\n",
                annotations_path()
            ),
        )
        .unwrap();
    }

    fn context() -> SemanticContext {
        SemanticContext {
            target_kind: "lib".into(),
            target_name: "payments".into(),
            crate_name: "payments".into(),
            module: vec!["capture".into()],
            manifest: PathBuf::from("Cargo.toml"),
        }
    }

    #[test]
    fn check_implementations_bind_to_exact_functions() {
        let markers = scan(
            "#[azimuth::realizes(\"polyglot/identity\", \"rust-identifies\")]\nfn identity() -> &'static str { \"rust\" }\n\n#[azimuth::implements_check(\"identity-check\")]\nfn first_test() { assert_eq!(identity(), \"rust\"); }\n\n#[azimuth::implements_check(\"identity-check\")]\nfn second_test() { assert!(!identity().is_empty()); }\n\nfn unmarked() {}\n",
            "service.rs",
            &context(),
        )
        .unwrap();

        assert_eq!(markers[0].site, "identity");
        assert_eq!(markers[1].site, "first_test");
        assert_eq!(markers[2].site, "second_test");
        let manifest = manifest_json(&markers);
        assert!(manifest.contains("\"check_implementations\""));
        assert!(manifest.contains("\"check\":\"identity-check\""));
        assert!(markers[1].fingerprint.starts_with("sha256:"));
        assert_eq!(markers[1].fingerprint.len(), 71);
        assert!(!manifest.contains("\"covers\""));
    }

    #[test]
    fn fingerprints_are_local_to_each_function() {
        let before = scan(
            "#[implements_check(\"check\")]\nfn first() { assert!(true); }\n\n#[implements_check(\"check\")]\nfn second() { assert!(true); }\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        let after = scan(
            "#[implements_check(\"check\")]\nfn first() { assert!(true); }\n\n#[implements_check(\"check\")]\nfn second() { assert!(false); }\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        assert_eq!(before[0].fingerprint, after[0].fingerprint);
        assert_ne!(before[1].fingerprint, after[1].fingerprint);
    }

    #[test]
    fn retired_attributes_fail_explicitly() {
        for marker in ["covers", "covers_mechanism"] {
            let error = scan(
                &format!("#[{marker}(\"a\", \"s\")]\nfn old() {{}}\n"),
                "service.rs",
                &context(),
            )
            .unwrap_err();
            assert!(error.contains(&format!("retired alpha 1 marker {marker}")));
        }
    }

    #[test]
    fn unrelated_qualified_covers_attribute_remains_ordinary() {
        let markers = scan(
            "#[other::covers(\"case\")]\nfn ordinary() {}\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        assert!(markers.is_empty());
    }

    #[test]
    fn implements_check_requires_one_literal() {
        let error = scan(
            "#[implements_check(\"a\", \"b\")]\nfn test_x() {}\n",
            "service.rs",
            &context(),
        )
        .unwrap_err();
        assert!(error.contains("needs exactly 1"));
    }

    #[test]
    fn mechanisms_use_crate_module_type_item_and_signature() {
        let markers = scan(
            "mod engine {\nstruct Worker;\nimpl Worker {\n#[implements_mechanism(\"payments/capture\", \"completion-guard\")]\nfn apply<T: Copy>(&self, value: T) -> Option<T> { Some(value) }\n}\n}\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        assert_eq!(markers.len(), 1);
        let marker = &markers[0];
        assert!(marker.site.starts_with(
            "cargo:lib:payments:payments::capture::engine::Worker::apply fn<$0 : Copy>"
        ));
        let manifest = manifest_json(&markers);
        assert!(manifest.contains(&format!("\"site\":\"{}\"", marker.site)));
        assert!(manifest.contains(&format!("\"binding\":\"rust-symbol:{}\"", marker.site)));
        assert!(!manifest.contains("service.rs#"));
    }

    #[test]
    fn trait_implementations_and_nested_items_do_not_collide() {
        let markers = scan(
            "trait First { fn apply(&self); }\ntrait Second { fn apply(&self); }\nstruct Worker;\nimpl First for Worker {\n#[implements_mechanism(\"alpha\", \"first\")]\nfn apply(&self) {}\n}\nimpl Second for Worker {\n#[implements_mechanism(\"alpha\", \"second\")]\nfn apply(&self) {}\n}\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        assert_eq!(markers.len(), 2);
        assert_ne!(markers[0].site, markers[1].site);
        assert!(markers
            .iter()
            .any(|marker| marker.site.contains("<Worker as First>")));
        assert!(markers
            .iter()
            .any(|marker| marker.site.contains("<Worker as Second>")));
    }

    #[test]
    fn relocation_preserves_semantic_site_and_fingerprint() {
        let source = "#[implements_mechanism(\"alpha\", \"guard\")]\nfn apply(value: u64) -> u64 { value }\n";
        let before = scan(source, "first.rs", &context()).unwrap();
        let after = scan(source, "second.rs", &context()).unwrap();
        assert_eq!(before[0].site, after[0].site);
        assert_eq!(before[0].fingerprint, after[0].fingerprint);
        assert_ne!(before[0].file, after[0].file);
        let mut relocated = after[0].clone();
        relocated.file = before[0].file.clone();
        assert_eq!(before[0], relocated);
    }

    #[test]
    fn mechanism_marker_remains_two_arguments() {
        let error = scan(
            "#[implements_mechanism(\"alpha\", \"guard\", \"extra\")]\nfn apply() {}\n",
            "service.rs",
            &context(),
        )
        .unwrap_err();
        assert!(error.contains("needs exactly 2"));
    }

    #[test]
    fn duplicate_qualified_site_fails_closed() {
        let markers = scan(
            "#[implements_mechanism(\"alpha\", \"first\")]\nfn apply() {}\n#[implements_mechanism(\"alpha\", \"second\")]\nfn apply() {}\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        let error = validate_mechanism_sites(&markers).unwrap_err();
        assert!(error.contains("ambiguous mechanism site"));
    }

    #[test]
    fn signatures_drop_patterns_and_canonicalize_generic_parameters() {
        let before = scan(
            "#[implements_mechanism(\"alpha\", \"guard\")]\nfn apply<T: Copy>(named: T, _: Option<T>) -> T where T: Send { named }\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        let after = scan(
            "#[implements_mechanism(\"alpha\", \"guard\")]\nfn apply<Value: Copy>(other: Value, ignored: Option<Value>) -> Value where Value: Send { other }\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        assert_eq!(before[0].site, after[0].site);
        assert!(!before[0].site.contains("named"));
        assert!(before[0].site.contains("fn<$0 : Copy>($0,Option < $0 >)"));
        assert!(before[0].site.contains("where $0 : Send"));
    }

    #[test]
    fn declared_type_path_spelling_remains_identity() {
        let alias = scan(
            "type Alias = u64;\n#[implements_mechanism(\"alpha\", \"guard\")]\nfn apply(value: Alias) {}\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        let underlying = scan(
            "type Alias = u64;\n#[implements_mechanism(\"alpha\", \"guard\")]\nfn apply(value: u64) {}\n",
            "service.rs",
            &context(),
        )
        .unwrap();
        assert_ne!(alias[0].site, underlying[0].site);
    }

    #[test]
    fn cargo_target_and_reachable_module_qualify_the_site() {
        let project = temporary_project("reachable");
        write_package(&project.0, "");
        fs::write(project.0.join("src/lib.rs"), "mod engine;\n").unwrap();
        let source = "use azimuth_annotations::implements_mechanism;\n#[implements_mechanism(\"alpha\", \"guard\")]\npub fn apply(value: u64) -> u64 { value }\n";
        let selected = project.0.join("src/engine.rs");
        fs::write(&selected, source).unwrap();

        let markers = emit(&[selected], &project.0).unwrap();
        assert_eq!(markers.len(), 1);
        assert!(markers[0]
            .site
            .starts_with("cargo:lib:sample:sample::engine::apply fn(u64)"));
        assert_eq!(markers[0].file, "src/engine.rs");
    }

    #[test]
    fn custom_unreachable_and_ambiguous_targets_fail_closed() {
        let custom = temporary_project("custom");
        write_package(&custom.0, "[lib]\npath = \"custom.rs\"");
        fs::write(custom.0.join("custom.rs"), "pub fn ordinary() {}\n").unwrap();
        let error = emit(&[custom.0.join("custom.rs")], &custom.0).unwrap_err();
        assert!(error.contains("unsupported custom source"), "{error}");

        let unreachable = temporary_project("unreachable");
        write_package(&unreachable.0, "");
        fs::write(unreachable.0.join("src/lib.rs"), "pub fn ordinary() {}\n").unwrap();
        fs::write(unreachable.0.join("src/orphan.rs"), "pub fn orphan() {}\n").unwrap();
        let error = emit(&[unreachable.0.join("src/orphan.rs")], &unreachable.0).unwrap_err();
        assert!(error.contains("unreachable"), "{error}");

        let ambiguous = temporary_project("ambiguous");
        write_package(&ambiguous.0, "");
        fs::write(ambiguous.0.join("src/lib.rs"), "mod shared;\n").unwrap();
        fs::write(
            ambiguous.0.join("src/main.rs"),
            "mod shared; fn main() {}\n",
        )
        .unwrap();
        fs::write(ambiguous.0.join("src/shared.rs"), "pub fn shared() {}\n").unwrap();
        let error = emit(&[ambiguous.0.join("src/shared.rs")], &ambiguous.0).unwrap_err();
        assert!(error.contains("several Cargo targets"), "{error}");
    }

    #[test]
    fn path_routing_and_compiler_rejection_fail_before_emission() {
        let routed = temporary_project("path");
        write_package(&routed.0, "");
        fs::write(
            routed.0.join("src/lib.rs"),
            "#[path = \"engine.rs\"] mod custom;\n",
        )
        .unwrap();
        fs::write(routed.0.join("src/engine.rs"), "pub fn ordinary() {}\n").unwrap();
        let error = emit(&[routed.0.join("src/engine.rs")], &routed.0).unwrap_err();
        assert!(error.contains("unsupported #[path]"), "{error}");

        let rejected = temporary_project("rejected");
        write_package(&rejected.0, "");
        let source = "use azimuth_annotations::implements_mechanism;\n#[implements_mechanism(\"alpha\", \"guard\")]\npub fn apply() -> Missing { panic!() }\n";
        fs::write(rejected.0.join("src/lib.rs"), source).unwrap();
        let error = emit(&[rejected.0.join("src/lib.rs")], &rejected.0).unwrap_err();
        assert!(error.contains("did not compile"), "{error}");
    }

    #[test]
    fn outside_root_has_no_absolute_fallback() {
        let root = temporary_project("root");
        write_package(&root.0, "");
        fs::write(root.0.join("src/lib.rs"), "pub fn ordinary() {}\n").unwrap();
        let outside = temporary_project("outside");
        write_package(&outside.0, "");
        fs::write(outside.0.join("src/lib.rs"), "pub fn ordinary() {}\n").unwrap();
        let error = emit(&[outside.0.join("src/lib.rs")], &root.0).unwrap_err();
        assert!(error.contains("outside --root"), "{error}");
    }

    #[test]
    fn duplicate_site_across_cargo_packages_fails_closed() {
        let root = temporary_project("package-collision");
        for (directory, mechanism) in [("first", "first"), ("second", "second")] {
            let package = root.0.join(directory);
            fs::create_dir_all(package.join("src")).unwrap();
            write_package(&package, "");
            fs::write(
                package.join("src/lib.rs"),
                format!(
                    "use azimuth_annotations::implements_mechanism;\n\
                     #[implements_mechanism(\"alpha\", \"{mechanism}\")]\n\
                     pub fn apply() {{}}\n"
                ),
            )
            .unwrap();
        }
        let error = emit(
            &[
                root.0.join("first/src/lib.rs"),
                root.0.join("second/src/lib.rs"),
            ],
            &root.0,
        )
        .unwrap_err();
        assert!(error.contains("ambiguous mechanism site"), "{error}");
    }

    #[test]
    fn sha256_matches_standard_vector() {
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
