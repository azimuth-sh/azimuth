//! The `azimuth` CLI.
//!
//! `azimuth` is the tool; `rtm` is one check among several (D9). The same binary owns deterministic
//! checking, change authoring and lifecycle gates, exploration discovery, and federated assembly.

use azimuth::check;
use azimuth::diag::Diag;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
azimuth — derives a model from claims and linkage tags, and flags its holes

USAGE
    azimuth check [<check-id>...] [options]
    azimuth export [options]
    azimuth judge [options]
    azimuth assurance export --project <id> --out <snapshot.json> [options]
    azimuth init [--root <azimuth-dir>]
    azimuth explore create <id> [--title <text>] [--explorations <dir>]
    azimuth explore list|show [<id>] [--explorations <dir>]
    azimuth project check --project <file> --workset <file> [--local <repository>]
    azimuth project export --project <file> --workset <file> [--local <repository>]
    azimuth project finalize --project <file> --workset <file> --out <snapshot.json>
    azimuth project accept-change --project <file> --before <workset> --after <workset>
        --change <id> --date <YYYY-MM-DD> --out <snapshot.json>
    azimuth project observe --project <file> --repository <id> --root <dir>
        --producer <name/version> --manifest <file>... --out <repository.json>
    azimuth project locate --reference <project-reference.json>
    azimuth change check <dir> [options]
    azimuth change create <id> [--title <text>] [--changes <dir>]
    azimuth change list [--changes <dir>]
    azimuth change show|status <id-or-dir> [--changes <dir>] [options]
    azimuth change work-packages <id-or-dir> [--changes <dir>]
    azimuth change instructions <id-or-dir> --package <id> [--changes <dir>]
    azimuth change finalize <dir> [options]
    azimuth change archive <dir> --date <YYYY-MM-DD> [options]

The judge command lists every claim with the fingerprint a judgment must carry, so the
agent tier can record verdicts that expire when what they judged changes.

CHECKS
    rtm     claims against the code and evidence that reference them

OPTIONS
    --model <dir>          current model packages (default: azimuth/model)
    --standards <file>     evidence standards (default: azimuth/standards/verification.md)
    --workspace <file>     areas, surfaces and obligations (default: workspace.json beside model/)
    --manifest <file>      a linkage manifest; repeatable
    --only <pattern>       restrict to spec ids; `billing/**` or an exact id; repeatable
    --out <file>           export destination (default: stdout)
    -h, --help
    -V, --version
";

const CHECKS: &[&str] = &["rtm"];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("azimuth: {message}");
            ExitCode::from(2)
        }
    }
}

struct Options {
    model: PathBuf,
    standards: PathBuf,
    workspace: PathBuf,
    manifests: Vec<PathBuf>,
    only: Vec<String>,
    out: Option<PathBuf>,
    checks: Vec<String>,
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    if args[0] == "-V" || args[0] == "--version" {
        println!("azimuth {}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::SUCCESS);
    }

    let command = args[0].clone();
    if command == "change" {
        return command_change(&args[1..]);
    }
    if command == "init" {
        return command_init(&args[1..]);
    }
    if command == "explore" {
        return command_explore(&args[1..]);
    }
    if command == "project" {
        return command_project(&args[1..]);
    }
    if command == "assurance" {
        return command_assurance(&args[1..]);
    }
    let options = parse_options(&args[1..])?;

    match command.as_str() {
        "check" => command_check(options),
        "export" => command_export(options),
        "judge" => command_judge(options),
        other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    }
}

fn command_assurance(args: &[String]) -> Result<ExitCode, String> {
    let Some(operation) = args.first() else {
        return Err("assurance needs export".into());
    };
    if operation != "export" {
        return Err(format!("unknown assurance operation `{operation}`"));
    }
    let mut project = None;
    let mut option_args = Vec::new();
    let mut index = 1;
    while index < args.len() {
        if args[index] == "--project" {
            project = Some(argument_value(args, index, "--project")?);
            index += 2;
        } else {
            option_args.push(args[index].clone());
            index += 1;
        }
    }
    let project = project.ok_or("assurance export needs `--project <id>`")?;
    azimuth::diag::validate_id(&project, false)
        .map_err(|error| format!("invalid assurance project id: {error}"))?;
    let options = parse_options(&option_args)?;
    if !options.only.is_empty() {
        return Err(
            "assurance export requires the complete accepted model; `--only` is unsupported".into(),
        );
    }
    let output = options
        .out
        .clone()
        .ok_or("assurance export needs `--out <snapshot.json>`")?;
    let loaded = match azimuth::load(
        &options.model,
        &options.standards,
        &options.workspace,
        &options.manifests,
        &options.only,
    ) {
        Ok(loaded) => loaded,
        Err(diags) => {
            report(&diags, "error");
            return Ok(ExitCode::from(2));
        }
    };
    report(&loaded.warnings, "warning");
    let holes = check::rtm(&loaded.model);
    let summary = check::summarize(&loaded.model, &holes);
    if summary.errors > 0 || summary.warnings > 0 || !loaded.warnings.is_empty() {
        eprintln!(
            "error: accepted model has {} error(s), {} warning(s)",
            summary.errors,
            summary.warnings + loaded.warnings.len()
        );
        return Ok(ExitCode::from(1));
    }
    let snapshot = azimuth::assurance::snapshot(&loaded.model, &holes, &project);
    std::fs::write(&output, snapshot.to_json().to_string_pretty())
        .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    println!(
        "exported assurance snapshot `{}` with {} claim contract(s)",
        snapshot.id,
        snapshot.claims.len()
    );
    Ok(ExitCode::SUCCESS)
}

fn command_init(args: &[String]) -> Result<ExitCode, String> {
    let root = match args {
        [] => PathBuf::from("azimuth"),
        [option, value] if option == "--root" => PathBuf::from(value),
        _ => return Err("init accepts only `--root <azimuth-dir>`".into()),
    };
    let created = azimuth::workflow::initialize(&root)?;
    if created.is_empty() {
        println!("Azimuth is already initialized at {}", root.display());
    } else {
        println!("initialized Azimuth at {}", root.display());
        for path in created {
            println!("  {}", path.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn command_explore(args: &[String]) -> Result<ExitCode, String> {
    let Some(operation) = args.first() else {
        return Err("explore needs create, list or show".into());
    };
    let mut explorations = PathBuf::from("azimuth/explorations");
    let mut positional = Vec::new();
    let mut title = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--explorations" => {
                explorations = PathBuf::from(argument_value(args, index, "--explorations")?);
                index += 2;
            }
            "--title" => {
                title = Some(argument_value(args, index, "--title")?);
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown explore option `{value}`"));
            }
            value => {
                positional.push(value.to_string());
                index += 1;
            }
        }
    }
    match operation.as_str() {
        "create" => {
            let id = positional.first().ok_or("explore create needs an id")?;
            if positional.len() != 1 {
                return Err("explore create accepts one id".into());
            }
            let root = azimuth::workflow::create_exploration(
                &explorations,
                id,
                title.as_deref().unwrap_or(id),
            )?;
            println!("created exploration `{id}` at {}", root.display());
            Ok(ExitCode::SUCCESS)
        }
        "list" => {
            if !positional.is_empty() {
                return Err("explore list accepts no id".into());
            }
            let entries = match std::fs::read_dir(&explorations) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(ExitCode::SUCCESS)
                }
                Err(error) => {
                    return Err(format!("cannot read {}: {error}", explorations.display()))
                }
            };
            let mut ids = entries
                .flatten()
                .filter(|entry| entry.path().join("exploration.md").is_file())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>();
            ids.sort();
            for id in ids {
                println!("{id}");
            }
            Ok(ExitCode::SUCCESS)
        }
        "show" => {
            let id = positional.first().ok_or("explore show needs an id")?;
            if positional.len() != 1 {
                return Err("explore show accepts one id".into());
            }
            let path = explorations.join(id).join("exploration.md");
            let source = std::fs::read_to_string(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            print!("{source}");
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown explore operation `{other}`")),
    }
}

fn argument_value(args: &[String], index: usize, name: &str) -> Result<String, String> {
    args.get(index + 1)
        .cloned()
        .ok_or_else(|| format!("`{name}` needs a value"))
}

struct ProjectOptions {
    project: PathBuf,
    workset: PathBuf,
    local: Option<String>,
    only: Vec<String>,
    out: Option<PathBuf>,
}

fn command_project(args: &[String]) -> Result<ExitCode, String> {
    let Some(operation) = args.first() else {
        return Err(format!("project needs an operation\n\n{USAGE}"));
    };
    if operation == "observe" {
        return command_project_observe(&args[1..]);
    }
    if operation == "locate" {
        return command_project_locate(&args[1..]);
    }
    if operation == "accept-change" {
        return command_project_accept_change(&args[1..]);
    }
    let mut project = None;
    let mut workset = None;
    let mut local = None;
    let mut only = Vec::new();
    let mut out = None;
    let mut index = 1;
    while index < args.len() {
        let value = |name: &str| {
            args.get(index + 1)
                .cloned()
                .ok_or_else(|| format!("`{name}` needs a value"))
        };
        match args[index].as_str() {
            "--project" => {
                project = Some(PathBuf::from(value("--project")?));
                index += 2;
            }
            "--workset" => {
                workset = Some(PathBuf::from(value("--workset")?));
                index += 2;
            }
            "--local" => {
                local = Some(value("--local")?);
                index += 2;
            }
            "--only" => {
                only.push(value("--only")?);
                index += 2;
            }
            "--out" => {
                out = Some(PathBuf::from(value("--out")?));
                index += 2;
            }
            other => return Err(format!("unknown project option `{other}`")),
        }
    }
    let options = ProjectOptions {
        project: project.ok_or("project command needs `--project <file>`")?,
        workset: workset.ok_or("project command needs `--workset <file>`")?,
        local,
        only,
        out,
    };
    let assembly = match azimuth::federation::assemble(
        &options.project,
        &options.workset,
        options.local.as_deref(),
    ) {
        Ok(assembly) => assembly,
        Err(diags) => {
            report(&diags, "error");
            eprintln!(
                "\n{} project assembly error(s); no model was derived",
                diags.len()
            );
            return Ok(ExitCode::from(2));
        }
    };
    let loaded = match azimuth::load_assembly(&assembly, &options.only) {
        Ok(loaded) => loaded,
        Err(diags) => {
            report(&diags, "error");
            return Ok(ExitCode::from(2));
        }
    };
    report(&loaded.warnings, "warning");
    let holes = check::rtm(&loaded.model);
    match operation.as_str() {
        "check" => {
            report_holes(&loaded.model, &holes, &["rtm"]);
            if assembly.complete {
                println!(
                    "project `{}` complete · {} repository input(s)",
                    assembly.project.id,
                    assembly.repositories.len()
                );
            } else {
                println!(
                    "local result for `{}` · project completeness: unknown",
                    assembly.local_repository.as_deref().unwrap_or("-")
                );
                if !assembly.missing_inputs.is_empty() {
                    println!(
                        "missing workset inputs: {}",
                        assembly.missing_inputs.join(", ")
                    );
                }
            }
            let summary = check::summarize(&loaded.model, &holes);
            Ok(if summary.errors > 0 {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "export" => {
            let json = loaded.model.to_json(&holes).to_string_pretty();
            match options.out {
                Some(path) => std::fs::write(&path, json)
                    .map_err(|error| format!("cannot write {}: {error}", path.display()))?,
                None => print!("{json}"),
            }
            Ok(ExitCode::SUCCESS)
        }
        "finalize" => {
            if options.local.is_some() || !assembly.complete {
                eprintln!("error: a partial project assembly cannot be finalized");
                return Ok(ExitCode::from(1));
            }
            let summary = check::summarize(&loaded.model, &holes);
            if summary.errors > 0 || summary.warnings > 0 || !loaded.warnings.is_empty() {
                eprintln!(
                    "error: project model has {} error(s), {} warning(s)",
                    summary.errors,
                    summary.warnings + loaded.warnings.len()
                );
                return Ok(ExitCode::from(1));
            }
            let Some(path) = options.out else {
                return Err("project finalize needs `--out <snapshot.json>`".into());
            };
            let snapshot = match assembly.snapshot_json() {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    eprintln!("error: {error}");
                    return Ok(ExitCode::from(1));
                }
            };
            std::fs::write(&path, snapshot)
                .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
            println!("finalized project `{}`", assembly.project.id,);
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown project operation `{other}`")),
    }
}

fn command_project_accept_change(args: &[String]) -> Result<ExitCode, String> {
    let mut project = None;
    let mut before = None;
    let mut after = None;
    let mut change = None;
    let mut date = None;
    let mut out = None;
    let mut index = 0;
    while index < args.len() {
        let value = |name: &str| {
            args.get(index + 1)
                .cloned()
                .ok_or_else(|| format!("`{name}` needs a value"))
        };
        match args[index].as_str() {
            "--project" => project = Some(PathBuf::from(value("--project")?)),
            "--before" => before = Some(PathBuf::from(value("--before")?)),
            "--after" => after = Some(PathBuf::from(value("--after")?)),
            "--change" => change = Some(value("--change")?),
            "--date" => date = Some(value("--date")?),
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            other => return Err(format!("unknown project accept-change option `{other}`")),
        }
        index += 2;
    }
    let project = project.ok_or("project accept-change needs `--project <file>`")?;
    let before = before.ok_or("project accept-change needs `--before <workset>`")?;
    let after = after.ok_or("project accept-change needs `--after <workset>`")?;
    let change = change.ok_or("project accept-change needs `--change <id>`")?;
    let date = date.ok_or("project accept-change needs `--date <YYYY-MM-DD>`")?;
    if !valid_date(&date) {
        return Err(format!(
            "invalid archive date `{date}`; expected YYYY-MM-DD"
        ));
    }
    let out = out.ok_or("project accept-change needs `--out <snapshot.json>`")?;
    let snapshot =
        match azimuth::federation::accept_change(&project, &before, &after, &change, &date) {
            Ok(snapshot) => snapshot,
            Err(diags) => {
                report(&diags, "error");
                return Ok(ExitCode::from(1));
            }
        };
    std::fs::write(&out, snapshot)
        .map_err(|error| format!("cannot write {}: {error}", out.display()))?;
    println!("accepted and archived `{change}` in project account");
    Ok(ExitCode::SUCCESS)
}

fn command_project_locate(args: &[String]) -> Result<ExitCode, String> {
    if args.len() != 2 || args[0] != "--reference" {
        return Err("project locate needs `--reference <project-reference.json>`".into());
    }
    let reference_path = PathBuf::from(&args[1]);
    let reference = match azimuth::federation::load_project_reference(&reference_path) {
        Ok(reference) => reference,
        Err(diags) => {
            report(&diags, "error");
            return Ok(ExitCode::from(2));
        }
    };
    let catalog = azimuth::federation::load_project(&reference.catalog).map_err(|diags| {
        diags
            .into_iter()
            .map(|diag| diag.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    println!("project: {}", reference.project);
    println!("repository: {}", reference.repository);
    println!("catalog: {}", reference.catalog.display());
    match &reference.workset {
        Some(workset) => println!("workset: {}", workset.display()),
        None => println!("workset: supplied by integration"),
    }
    let areas = catalog
        .areas
        .iter()
        .filter(|area| area.repository == reference.repository)
        .map(|area| area.id.as_str())
        .collect::<Vec<_>>();
    let model_sources = catalog
        .model_sources
        .iter()
        .filter(|source| source.repository == reference.repository)
        .map(|source| format!("{}:{}", source.id, source.path))
        .collect::<Vec<_>>();
    println!("areas: {}", display_values(&areas));
    println!("model sources: {}", display_values(&model_sources));
    Ok(ExitCode::SUCCESS)
}

fn display_values<T: std::fmt::Display>(values: &[T]) -> String {
    if values.is_empty() {
        "none".into()
    } else {
        values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn command_project_observe(args: &[String]) -> Result<ExitCode, String> {
    let mut project = None;
    let mut repository = None;
    let mut root = None;
    let mut producer = None;
    let mut manifests = Vec::new();
    let mut out = None;
    let mut index = 0;
    while index < args.len() {
        let value = |name: &str| {
            args.get(index + 1)
                .cloned()
                .ok_or_else(|| format!("`{name}` needs a value"))
        };
        match args[index].as_str() {
            "--project" => project = Some(PathBuf::from(value("--project")?)),
            "--repository" => repository = Some(value("--repository")?),
            "--root" => root = Some(PathBuf::from(value("--root")?)),
            "--producer" => producer = Some(value("--producer")?),
            "--manifest" => manifests.push(PathBuf::from(value("--manifest")?)),
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            other => return Err(format!("unknown project observe option `{other}`")),
        }
        index += 2;
    }
    let project = project.ok_or("project observe needs `--project <file>`")?;
    let repository = repository.ok_or("project observe needs `--repository <id>`")?;
    let root = root.ok_or("project observe needs `--root <dir>`")?;
    let producer = producer.ok_or("project observe needs `--producer <name/version>`")?;
    let out = out.ok_or("project observe needs `--out <repository.json>`")?;
    let observation = match azimuth::federation::observe_repository(
        &project,
        &repository,
        &root,
        &producer,
        &manifests,
    ) {
        Ok(observation) => observation,
        Err(diags) => {
            report(&diags, "error");
            return Ok(ExitCode::from(2));
        }
    };
    std::fs::write(&out, observation)
        .map_err(|error| format!("cannot write {}: {error}", out.display()))?;
    println!("observed repository `{repository}` as {}", out.display());
    Ok(ExitCode::SUCCESS)
}

fn command_change(args: &[String]) -> Result<ExitCode, String> {
    let Some(operation) = args.first() else {
        return Err(format!("change needs an operation\n\n{USAGE}"));
    };
    let mut changes = PathBuf::from("azimuth/changes");
    let mut title = None;
    let mut package = None;
    let mut option_args = Vec::new();
    let mut positional = Vec::new();
    let mut date = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--date" => {
                date = Some(argument_value(args, index, "--date")?);
                index += 2;
            }
            "--changes" => {
                changes = PathBuf::from(argument_value(args, index, "--changes")?);
                index += 2;
            }
            "--title" => {
                title = Some(argument_value(args, index, "--title")?);
                index += 2;
            }
            "--package" => {
                package = Some(argument_value(args, index, "--package")?);
                index += 2;
            }
            value if value.starts_with('-') => {
                option_args.push(value.to_string());
                if ["--model", "--standards", "--manifest", "--only", "--out"].contains(&value) {
                    option_args.push(argument_value(args, index, value)?);
                    index += 2;
                } else {
                    index += 1;
                }
            }
            value => {
                positional.push(value.to_string());
                index += 1;
            }
        }
    }

    match operation.as_str() {
        "create" => {
            let id = one_position(&positional, "change create needs one id")?;
            let root =
                azimuth::workflow::create_change(&changes, id, title.as_deref().unwrap_or(id))?;
            println!("created change `{id}` at {}", root.display());
            return Ok(ExitCode::SUCCESS);
        }
        "list" => {
            if !positional.is_empty() {
                return Err("change list accepts no id".into());
            }
            for summary in azimuth::workflow::list_changes(&changes)? {
                println!(
                    "{}\t{}\t{}\t{}",
                    summary.id,
                    if summary.archived {
                        "archived"
                    } else {
                        "active"
                    },
                    summary.status,
                    summary.path.display()
                );
            }
            return Ok(ExitCode::SUCCESS);
        }
        "show" => {
            let value = one_position(&positional, "change show needs one id or directory")?;
            let root = azimuth::workflow::resolve_change(&changes, value)?;
            print!("{}", azimuth::workflow::render_change(&root)?);
            return Ok(ExitCode::SUCCESS);
        }
        "work-packages" => {
            let value = one_position(
                &positional,
                "change work-packages needs one id or directory",
            )?;
            let root = azimuth::workflow::resolve_change(&changes, value)?;
            let packages =
                azimuth::workflow::load_work_packages(&root).map_err(|errors| errors.join("\n"))?;
            let eligible = azimuth::workflow::eligible_packages(&packages)
                .into_iter()
                .map(|item| item.id.clone())
                .collect::<std::collections::BTreeSet<_>>();
            for item in packages {
                println!(
                    "{}\t{}\t{}\t{}",
                    item.id,
                    item.status.name(),
                    if eligible.contains(item.id.as_str()) {
                        "eligible"
                    } else {
                        "waiting"
                    },
                    if item.depends_on.is_empty() {
                        "none".into()
                    } else {
                        item.depends_on.join(",")
                    }
                );
            }
            return Ok(ExitCode::SUCCESS);
        }
        "instructions" => {
            let value = one_position(&positional, "change instructions needs one id or directory")?;
            let package = package.ok_or("change instructions needs `--package <id>`")?;
            let root = azimuth::workflow::resolve_change(&changes, value)?;
            let packages =
                azimuth::workflow::load_work_packages(&root).map_err(|errors| errors.join("\n"))?;
            let selected = packages
                .iter()
                .find(|item| item.id == package)
                .ok_or_else(|| format!("unknown work package `{package}`"))?;
            if !azimuth::workflow::eligible_packages(&packages)
                .iter()
                .any(|item| item.id == package)
            {
                return Err(format!("work package `{package}` is not eligible"));
            }
            print!(
                "{}",
                azimuth::workflow::package_instructions(&root, selected)
            );
            return Ok(ExitCode::SUCCESS);
        }
        _ => {}
    }

    let value = one_position(&positional, "change operation needs one id or directory")?;
    let root = azimuth::workflow::resolve_change(&changes, value)?;
    let options = parse_options(&option_args)?;
    let loaded = match azimuth::load(
        &options.model,
        &options.standards,
        &options.workspace,
        &options.manifests,
        &options.only,
    ) {
        Ok(loaded) => loaded,
        Err(diags) => {
            report(&diags, "error");
            return Ok(ExitCode::from(2));
        }
    };
    let report = match azimuth::change::inspect(&root, &loaded.model) {
        Ok(report) => report,
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            return Ok(ExitCode::from(2));
        }
    };
    let holes = check::rtm(&loaded.model);

    match operation.as_str() {
        "check" | "status" => {
            println!("change `{}`", report.id);
            for addition in &report.additions {
                let state = if addition.applied {
                    "applied"
                } else {
                    "planned"
                };
                println!(
                    "  add {}#{} · {} · {} scenario(s) · {state} · {}",
                    addition.spec,
                    addition.requirement,
                    addition.criticality.name(),
                    addition.scenarios.len(),
                    change_obligations(addition.criticality)
                );
            }
            for change in &report.criticality_changes {
                let state = if change.applied { "applied" } else { "planned" };
                println!(
                    "  criticality {}#{} · {} → {} · {state} · {}",
                    change.spec,
                    change.requirement,
                    change.from.name(),
                    change.to.name(),
                    change_obligations(change.to)
                );
            }
            println!(
                "current {} claim(s) → target {} claim(s)",
                report.current_claims, report.target_claims
            );
            println!("{} incomplete plan item(s)", report.incomplete_plan_items);
            let summary = check::summarize(&loaded.model, &holes);
            println!(
                "accepted-state model: {} error(s), {} warning(s)",
                summary.errors, summary.warnings
            );
            Ok(if summary.errors > 0 {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "finalize" => {
            let issues = azimuth::change::completion_issues(&root, &report);
            let summary = check::summarize(&loaded.model, &holes);
            if summary.errors > 0 || summary.warnings > 0 {
                eprintln!(
                    "error: accepted-state model has {} error(s), {} warning(s)",
                    summary.errors, summary.warnings
                );
            }
            for issue in &issues {
                eprintln!("error: {issue}");
            }
            if summary.errors > 0 || summary.warnings > 0 || !issues.is_empty() {
                return Ok(ExitCode::from(1));
            }
            let (fingerprint, finalization) = azimuth::change::finalization(&loaded.model, &holes);
            std::fs::write(root.join("finalization.json"), finalization).map_err(|error| {
                format!("cannot write {}/finalization.json: {error}", root.display())
            })?;
            println!("finalized `{}` at model {fingerprint}", report.id);
            Ok(ExitCode::SUCCESS)
        }
        "archive" => {
            let Some(date) = date else {
                return Err("change archive needs `--date <YYYY-MM-DD>`".into());
            };
            if !valid_date(&date) {
                return Err(format!(
                    "invalid archive date `{date}`; expected YYYY-MM-DD"
                ));
            }
            let issues = azimuth::change::completion_issues(&root, &report);
            if !issues.is_empty() {
                for issue in &issues {
                    eprintln!("error: {issue}");
                }
                return Ok(ExitCode::from(1));
            }
            let finalization_path = root.join("finalization.json");
            let recorded = std::fs::read_to_string(&finalization_path).map_err(|_| {
                format!(
                    "{} is missing; run `azimuth change finalize` first",
                    finalization_path.display()
                )
            })?;
            let (_, expected) = azimuth::change::finalization(&loaded.model, &holes);
            if recorded != expected {
                return Ok({
                    eprintln!("error: finalization is stale; run `azimuth change finalize` again");
                    ExitCode::from(1)
                });
            }
            if !holes.is_empty() {
                eprintln!("error: accepted-state model has holes");
                return Ok(ExitCode::from(1));
            }
            if root
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str())
                != Some("changes")
            {
                return Err("archive source must be a direct child of `changes/`".into());
            }
            let archive_root = root.parent().unwrap().join("archive");
            std::fs::create_dir_all(&archive_root)
                .map_err(|error| format!("cannot create {}: {error}", archive_root.display()))?;
            let destination = archive_root.join(format!("{date}-{}", report.id));
            if destination.exists() {
                return Err(format!(
                    "archive destination {} already exists",
                    destination.display()
                ));
            }
            std::fs::rename(&root, &destination).map_err(|error| {
                format!(
                    "cannot archive {} as {}: {error}",
                    root.display(),
                    destination.display()
                )
            })?;
            println!("archived `{}` as {}", report.id, destination.display());
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown change operation `{other}`")),
    }
}

fn one_position<'a>(values: &'a [String], error: &str) -> Result<&'a str, String> {
    match values {
        [value] => Ok(value),
        _ => Err(error.into()),
    }
}

fn change_obligations(criticality: azimuth::model::Criticality) -> &'static str {
    match criticality {
        azimuth::model::Criticality::Routine => "intent only",
        azimuth::model::Criticality::Standard => "realization + evidence",
        azimuth::model::Criticality::Critical => {
            "realization + design + critical evidence + judgment"
        }
    }
}

fn valid_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        model: PathBuf::from("azimuth/model"),
        standards: PathBuf::from("azimuth/standards/verification.md"),
        workspace: PathBuf::new(),
        manifests: Vec::new(),
        only: Vec::new(),
        out: None,
        checks: Vec::new(),
    };
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let value = |name: &str| -> Result<String, String> {
            args.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("`{name}` needs a value"))
        };
        match arg.as_str() {
            "--model" => {
                o.model = PathBuf::from(value("--model")?);
                i += 2;
            }
            "--standards" => {
                o.standards = PathBuf::from(value("--standards")?);
                i += 2;
            }
            "--workspace" => {
                o.workspace = PathBuf::from(value("--workspace")?);
                i += 2;
            }
            "--manifest" => {
                o.manifests.push(PathBuf::from(value("--manifest")?));
                i += 2;
            }
            "--only" => {
                o.only.push(value("--only")?);
                i += 2;
            }
            "--out" => {
                o.out = Some(PathBuf::from(value("--out")?));
                i += 2;
            }
            other if other.starts_with('-') => return Err(format!("unknown option `{other}`")),
            other => {
                if !CHECKS.contains(&other) {
                    return Err(format!(
                        "unknown check `{other}`\n  known checks: {}",
                        CHECKS.join(", ")
                    ));
                }
                o.checks.push(other.to_string());
                i += 1;
            }
        }
    }
    if o.workspace.as_os_str().is_empty() {
        o.workspace = o
            .model
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("workspace.json");
    }
    Ok(o)
}

fn report(diags: &[Diag], label: &str) {
    for d in diags {
        eprintln!("{label}: {d}");
    }
}

fn report_holes(model: &azimuth::model::Model, holes: &[check::Hole], selected: &[&str]) {
    for hole in holes {
        let where_ = if hole.line > 0 {
            format!("{}:{}", hole.path, hole.line)
        } else {
            hole.path.clone()
        };
        let claim = hole.claim.clone().unwrap_or_else(|| "-".into());
        let level = hole
            .criticality
            .map(|criticality| format!(" ({})", criticality.name()))
            .unwrap_or_default();
        println!(
            "{where_}: {} {} {claim}{level}\n    {}",
            hole.severity.name(),
            hole.kind.name(),
            hole.detail
        );
    }
    let summary = check::summarize(model, holes);
    let by_kind = check::counts_by_kind(holes)
        .into_iter()
        .map(|(kind, count)| format!("{count} {kind}"))
        .collect::<Vec<_>>();
    println!();
    println!(
        "{} claims in {} spec(s) · checks: {}",
        summary.claims,
        model.specs.len(),
        selected.join(", ")
    );
    if by_kind.is_empty() {
        println!("no holes");
    } else {
        println!("{}", by_kind.join(" · "));
    }
    println!(
        "{} error(s), {} warning(s)",
        summary.errors, summary.warnings
    );
}

fn command_check(options: Options) -> Result<ExitCode, String> {
    let loaded = match azimuth::load(
        &options.model,
        &options.standards,
        &options.workspace,
        &options.manifests,
        &options.only,
    ) {
        Ok(l) => l,
        Err(diags) => {
            report(&diags, "error");
            eprintln!("\n{} parse error(s); no model was derived", diags.len());
            return Ok(ExitCode::from(2));
        }
    };
    report(&loaded.warnings, "warning");

    let selected: Vec<&str> = if options.checks.is_empty() {
        CHECKS.to_vec()
    } else {
        options.checks.iter().map(|s| s.as_str()).collect()
    };

    let mut holes = Vec::new();
    for id in &selected {
        match *id {
            "rtm" => holes.extend(check::rtm(&loaded.model)),
            _ => unreachable!("checked during option parsing"),
        }
    }

    for hole in &holes {
        let where_ = if hole.line > 0 {
            format!("{}:{}", hole.path, hole.line)
        } else {
            hole.path.clone()
        };
        let claim = hole.claim.clone().unwrap_or_else(|| "-".into());
        let level = hole
            .criticality
            .map(|c| format!(" ({})", c.name()))
            .unwrap_or_default();
        println!(
            "{where_}: {} {} {claim}{level}\n    {}",
            hole.severity.name(),
            hole.kind.name(),
            hole.detail
        );
    }

    let summary = check::summarize(&loaded.model, &holes);
    let by_kind: Vec<String> = check::counts_by_kind(&holes)
        .into_iter()
        .map(|(k, n)| format!("{n} {k}"))
        .collect();

    println!();
    println!(
        "{} claims in {} spec(s) · checks: {}",
        summary.claims,
        loaded.model.specs.len(),
        selected.join(", ")
    );
    if by_kind.is_empty() {
        println!("no holes");
    } else {
        println!("{}", by_kind.join(" · "));
    }
    println!(
        "{} error(s), {} warning(s)",
        summary.errors, summary.warnings
    );

    Ok(if summary.errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// Lists claims and their current fingerprints — the agent tier's worklist.
fn command_judge(options: Options) -> Result<ExitCode, String> {
    let loaded = match azimuth::load(
        &options.model,
        &options.standards,
        &options.workspace,
        &options.manifests,
        &options.only,
    ) {
        Ok(l) => l,
        Err(diags) => {
            report(&diags, "error");
            return Ok(ExitCode::from(2));
        }
    };
    let model = &loaded.model;

    for claim in model.claims() {
        let inputs = model.judgment_inputs(&claim.spec.id, &claim.scenario.id);
        let fingerprint = azimuth::judgment::fingerprint(&model.claim_text(&claim), inputs.clone());
        let existing = model
            .judgments_for(&claim.spec.id)
            .and_then(|j| j.entry(&claim.scenario.id));
        let state = match existing {
            Some(j) if j.fingerprint == fingerprint => format!("judged {}", j.verdict.name()),
            Some(j) => format!("stale ({})", j.verdict.name()),
            None => "unjudged".to_string(),
        };
        println!(
            "{}\t{}\t{}\t{}\t{}",
            claim.spec.id,
            claim.scenario.id,
            claim
                .requirement
                .criticality
                .map(|c| c.name())
                .unwrap_or("-"),
            fingerprint,
            state
        );
        for input in inputs {
            println!("\t{}\t{}", input.role(), input.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn command_export(options: Options) -> Result<ExitCode, String> {
    let loaded = match azimuth::load(
        &options.model,
        &options.standards,
        &options.workspace,
        &options.manifests,
        &options.only,
    ) {
        Ok(l) => l,
        Err(diags) => {
            report(&diags, "error");
            return Ok(ExitCode::from(2));
        }
    };
    report(&loaded.warnings, "warning");

    let holes = check::rtm(&loaded.model);
    let json = loaded.model.to_json(&holes).to_string_pretty();

    match options.out {
        Some(path) => std::fs::write(&path, json)
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?,
        None => print!("{json}"),
    }
    Ok(ExitCode::SUCCESS)
}
