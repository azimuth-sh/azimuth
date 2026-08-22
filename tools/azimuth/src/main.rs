//! The `azimuth` CLI.
//!
//! The same binary owns deterministic validation and reporting, change authoring and lifecycle
//! gates, exploration discovery, and federated assembly.

use azimuth::diag::Diag;
use azimuth::validation;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};

static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

const USAGE: &str = "\
azimuth — derives and validates an evidence-control-plane model

USAGE
    azimuth validate [options]
    azimuth report traceability [options]
    azimuth export [options]
    azimuth adapter verify [--config <file>]
    azimuth run plan --request <file> [--model <dir>] [--standards <file>]
        [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]
    azimuth run execute --plan <file> [--predecessor <bundle>...] [--config <file>] [--out <file>]
    azimuth run import --plan <file> --input <id>=<file>... [--predecessor <bundle>...]
        [--config <file>] [--out <file>]
    azimuth run verify --bundle <file>...
    azimuth run inspect --bundle <file>... [--format text|json] [--out <file>]
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

OPTIONS
    --model <dir>          current model packages (default: azimuth/model)
    --standards <file>     decision policies and Challenge schedule
                           (default: azimuth/standards/verification.md)
    --workspace <file>     areas, surfaces and obligations (default: workspace.json beside model/)
    --manifest <file>      a linkage manifest; repeatable
    --only <pattern>       restrict to spec ids; `billing/**` or an exact id; repeatable
    --out <file>           export destination (default: stdout)
    -h, --help
    -V, --version

RUN PLAN REQUESTS
    The strict request file may select Checks, Challenges, or both. See
    contracts/run-launch-plan.md for the complete request shape.
";

const RUN_USAGE: &str = "\
USAGE
    azimuth run plan --request <file> [--model <dir>] [--standards <file>]
        [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]
    azimuth run execute --plan <file> [--predecessor <bundle>...]
        [--config <file>] [--out <file>]
    azimuth run import --plan <file> --input <id>=<file>...
        [--predecessor <bundle>...] [--config <file>] [--out <file>]
    azimuth run verify --bundle <file>...
    azimuth run inspect --bundle <file>... [--format text|json] [--out <file>]

PLAN REQUESTS
    The strict request file may select Checks, Challenges, or both. See
    contracts/run-launch-plan.md for the complete request shape.
";

const ADAPTER_USAGE: &str = "\
USAGE
    azimuth adapter verify [--config <file>]
";

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
    if command == "report" {
        return command_report(&args[1..]);
    }
    if command == "run" {
        return command_run(&args[1..]);
    }
    if command == "adapter" {
        return command_adapter(&args[1..]);
    }

    match command.as_str() {
        "validate" => command_validate(parse_options(&args[1..])?),
        "export" => command_export(parse_options(&args[1..])?),
        other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    }
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
    println!(
        "next: azimuth validate --model {} --standards {} --workspace {}",
        root.join("model").display(),
        root.join("standards/verification.md").display(),
        root.join("workspace.json").display()
    );
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
    let findings = validation::validate(&loaded.model);
    match operation.as_str() {
        "check" => {
            report_findings(&loaded.model, &findings);
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
            let summary = validation::summarize(&loaded.model, &findings);
            Ok(if summary.errors > 0 {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "export" => {
            let json = loaded.model.to_json(&findings).to_string_pretty();
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
            let summary = validation::summarize(&loaded.model, &findings);
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
    let findings = validation::validate(&loaded.model);

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
            if let Some(reason) = &report.unchanged_intent_reason {
                println!("  intent unchanged · {reason}");
            }
            println!(
                "current {} claim(s) → target {} claim(s)",
                report.current_claims, report.target_claims
            );
            println!("{} incomplete plan item(s)", report.incomplete_plan_items);
            let summary = validation::summarize(&loaded.model, &findings);
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
            let summary = validation::summarize(&loaded.model, &findings);
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
            let (fingerprint, finalization) =
                azimuth::change::finalization(&loaded.model, &findings);
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
            let (_, expected) = azimuth::change::finalization(&loaded.model, &findings);
            if recorded != expected {
                return Ok({
                    eprintln!("error: finalization is stale; run `azimuth change finalize` again");
                    ExitCode::from(1)
                });
            }
            if !findings.is_empty() {
                eprintln!("error: accepted-state model has findings");
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
        azimuth::model::Criticality::Standard => "realization + qualified verification",
        azimuth::model::Criticality::Critical => "realization + design + qualified verification",
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

#[derive(Clone, Copy)]
enum RunOutputFormat {
    Text,
    Json,
}

struct RunOptions {
    bundles: Vec<PathBuf>,
    format: RunOutputFormat,
    out: Option<PathBuf>,
}

struct RunPlanOptions {
    request: PathBuf,
    model: PathBuf,
    standards: PathBuf,
    workspace: PathBuf,
    manifests: Vec<PathBuf>,
    config: PathBuf,
    out: Option<PathBuf>,
}

struct RunInvokeOptions {
    plan: PathBuf,
    predecessors: Vec<PathBuf>,
    inputs: Vec<azimuth::adapter_host::ImportInput>,
    config: PathBuf,
    out: Option<PathBuf>,
}

fn command_adapter(args: &[String]) -> Result<ExitCode, String> {
    let Some(operation) = args.first() else {
        return Err(format!("adapter needs verify\n\n{ADAPTER_USAGE}"));
    };
    if operation == "-h" || operation == "--help" {
        print!("{ADAPTER_USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    if operation != "verify" {
        return Err(format!(
            "unknown adapter operation `{operation}`\n\n{ADAPTER_USAGE}"
        ));
    }
    if args
        .get(1)
        .is_some_and(|value| value == "-h" || value == "--help")
    {
        if args.len() == 2 {
            print!("{ADAPTER_USAGE}");
            return Ok(ExitCode::SUCCESS);
        }
    }
    let mut config = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--config" => {
                set_once_path(
                    &mut config,
                    PathBuf::from(argument_value(args, index, "--config")?),
                    "--config",
                )?;
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown adapter verify option `{value}`"));
            }
            value => return Err(format!("unexpected adapter positional argument `{value}`")),
        }
    }
    let config = config.unwrap_or_else(default_adapter_configuration);
    let configuration = match azimuth::adapter::load_configuration(&config) {
        Ok(configuration) => configuration,
        Err(errors) => return Ok(report_schema_errors(&errors)),
    };
    for adapter in &configuration.adapters {
        if let Err(error) = azimuth::adapter_host::verify_adapter(adapter) {
            report_host_error(&error);
            return Ok(ExitCode::from(error.class.exit_code() as u8));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn command_run(args: &[String]) -> Result<ExitCode, String> {
    let Some(operation) = args.first() else {
        return Err(format!(
            "run needs plan, execute, import, verify or inspect\n\n{RUN_USAGE}"
        ));
    };
    if operation == "-h" || operation == "--help" {
        print!("{RUN_USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    if args
        .get(1)
        .is_some_and(|argument| argument == "-h" || argument == "--help")
    {
        if matches!(
            operation.as_str(),
            "plan" | "execute" | "import" | "verify" | "inspect"
        ) && args.len() == 2
        {
            print!("{RUN_USAGE}");
            return Ok(ExitCode::SUCCESS);
        }
    }
    match operation.as_str() {
        "plan" => command_run_plan(parse_run_plan_options(&args[1..])?),
        "execute" => command_run_invoke(parse_run_invoke_options(&args[1..], false)?, false),
        "import" => command_run_invoke(parse_run_invoke_options(&args[1..], true)?, true),
        "verify" => command_run_verify(parse_run_options(&args[1..], false)?),
        "inspect" => command_run_inspect(parse_run_options(&args[1..], true)?),
        other => Err(format!("unknown run operation `{other}`\n\n{RUN_USAGE}")),
    }
}

fn parse_run_plan_options(args: &[String]) -> Result<RunPlanOptions, String> {
    let mut request = None;
    let mut model = None;
    let mut standards = None;
    let mut workspace = None;
    let mut manifests = Vec::new();
    let mut config = None;
    let mut out = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index].as_str();
        match option {
            "--request" => set_once_path(
                &mut request,
                PathBuf::from(argument_value(args, index, option)?),
                option,
            )?,
            "--model" => set_once_path(
                &mut model,
                PathBuf::from(argument_value(args, index, option)?),
                option,
            )?,
            "--standards" => set_once_path(
                &mut standards,
                PathBuf::from(argument_value(args, index, option)?),
                option,
            )?,
            "--workspace" => set_once_path(
                &mut workspace,
                PathBuf::from(argument_value(args, index, option)?),
                option,
            )?,
            "--manifest" => manifests.push(PathBuf::from(argument_value(args, index, option)?)),
            "--config" => set_once_path(
                &mut config,
                PathBuf::from(argument_value(args, index, option)?),
                option,
            )?,
            "--out" => set_once_path(
                &mut out,
                PathBuf::from(argument_value(args, index, option)?),
                option,
            )?,
            value if value.starts_with('-') => {
                return Err(format!("unknown run plan option `{value}`"));
            }
            value => return Err(format!("unexpected run plan positional argument `{value}`")),
        }
        index += 2;
    }
    let request = request.ok_or("run plan needs `--request <file>`")?;
    let model = model.unwrap_or_else(|| PathBuf::from("azimuth/model"));
    let standards = standards.unwrap_or_else(|| PathBuf::from("azimuth/standards/verification.md"));
    let workspace = workspace.unwrap_or_else(|| {
        model
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("workspace.json")
    });
    let config = config.unwrap_or_else(default_adapter_configuration);
    let input_paths = std::iter::once(&request)
        .chain(std::iter::once(&model))
        .chain(std::iter::once(&standards))
        .chain(std::iter::once(&workspace))
        .chain(manifests.iter())
        .chain(std::iter::once(&config));
    reject_output_input_equality(out.as_ref(), input_paths, "run plan")?;
    reject_model_output_descendant(out.as_ref(), &model)?;
    Ok(RunPlanOptions {
        request,
        model,
        standards,
        workspace,
        manifests,
        config,
        out,
    })
}

fn parse_run_invoke_options(args: &[String], import: bool) -> Result<RunInvokeOptions, String> {
    let operation = if import { "import" } else { "execute" };
    let mut plan = None;
    let mut predecessors = Vec::new();
    let mut inputs = Vec::new();
    let mut input_ids = BTreeSet::new();
    let mut config = None;
    let mut out = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index].as_str();
        match option {
            "--plan" => set_once_path(
                &mut plan,
                PathBuf::from(argument_value(args, index, option)?),
                option,
            )?,
            "--predecessor" => {
                predecessors.push(PathBuf::from(argument_value(args, index, option)?));
            }
            "--input" if import => {
                let value = argument_value(args, index, option)?;
                let (id, path) = value
                    .split_once('=')
                    .ok_or("`--input` needs `<lower-kebab-path-id>=<file>`")?;
                if id.is_empty() || path.is_empty() {
                    return Err("`--input` needs `<lower-kebab-path-id>=<file>`".into());
                }
                if !input_ids.insert(id.to_string()) {
                    return Err(format!("duplicate import input id `{id}`"));
                }
                inputs.push(azimuth::adapter_host::ImportInput {
                    id: id.to_string(),
                    path: PathBuf::from(path),
                });
            }
            "--config" => set_once_path(
                &mut config,
                PathBuf::from(argument_value(args, index, option)?),
                option,
            )?,
            "--out" => set_once_path(
                &mut out,
                PathBuf::from(argument_value(args, index, option)?),
                option,
            )?,
            value if value.starts_with('-') => {
                return Err(format!("unknown run {operation} option `{value}`"));
            }
            value => {
                return Err(format!(
                    "unexpected run {operation} positional argument `{value}`"
                ));
            }
        }
        index += 2;
    }
    let plan = plan.ok_or_else(|| format!("run {operation} needs `--plan <file>`"))?;
    if import && inputs.is_empty() {
        return Err("run import needs at least one `--input <id>=<file>`".into());
    }
    inputs.sort_by(|left, right| left.id.cmp(&right.id));
    let config = config.unwrap_or_else(default_adapter_configuration);
    let input_paths = std::iter::once(&plan)
        .chain(predecessors.iter())
        .chain(inputs.iter().map(|input| &input.path))
        .chain(std::iter::once(&config));
    reject_output_input_equality(out.as_ref(), input_paths, &format!("run {operation}"))?;
    Ok(RunInvokeOptions {
        plan,
        predecessors,
        inputs,
        config,
        out,
    })
}

fn command_run_plan(options: RunPlanOptions) -> Result<ExitCode, String> {
    let request = match azimuth::run_plan::load_plan_request(&options.request) {
        Ok(request) => request,
        Err(errors) => return Ok(report_schema_errors(&errors)),
    };
    let configuration = match azimuth::adapter::load_configuration(&options.config) {
        Ok(configuration) => configuration,
        Err(errors) => return Ok(report_schema_errors(&errors)),
    };
    let loaded = match azimuth::load(
        &options.model,
        &options.standards,
        &options.workspace,
        &options.manifests,
        &[],
    ) {
        Ok(loaded) => loaded,
        Err(diags) => {
            report(&diags, "error");
            return Ok(ExitCode::from(2));
        }
    };
    report(&loaded.warnings, "warning");
    let launch = match azimuth::run_plan::plan(&loaded.model, &configuration, &request) {
        Ok(launch) => launch,
        Err(errors) => {
            for error in errors {
                eprintln!("error: {error}");
            }
            return Ok(ExitCode::from(1));
        }
    };
    publish_output(
        azimuth::run_plan::launch_plan_to_json(&launch)
            .to_string_pretty()
            .as_bytes(),
        options.out.as_ref(),
    )?;
    Ok(ExitCode::SUCCESS)
}

fn command_run_invoke(options: RunInvokeOptions, import: bool) -> Result<ExitCode, String> {
    let launch = match azimuth::run_plan::load_launch_plan(&options.plan) {
        Ok(launch) => launch,
        Err(errors) => return Ok(report_schema_errors(&errors)),
    };
    let configuration = match azimuth::adapter::load_configuration(&options.config) {
        Ok(configuration) => configuration,
        Err(errors) => return Ok(report_schema_errors(&errors)),
    };
    let predecessors = match load_run_bundles(&options.predecessors) {
        Ok(bundles) => bundles,
        Err(code) => return Ok(code),
    };
    let hosted = if import {
        azimuth::adapter_host::import(&configuration, &launch, &options.inputs, &predecessors)
    } else {
        azimuth::adapter_host::execute(&configuration, &launch, &predecessors)
    };
    let hosted = match hosted {
        Ok(hosted) => hosted,
        Err(error) => {
            report_host_error(&error);
            return Ok(ExitCode::from(error.class.exit_code() as u8));
        }
    };
    publish_output(hosted.canonical_json.as_bytes(), options.out.as_ref())?;
    Ok(ExitCode::SUCCESS)
}

fn default_adapter_configuration() -> PathBuf {
    PathBuf::from("azimuth/adapters.json")
}

fn set_once_path(target: &mut Option<PathBuf>, value: PathBuf, option: &str) -> Result<(), String> {
    if target.replace(value).is_some() {
        Err(format!("`{option}` may be supplied only once"))
    } else {
        Ok(())
    }
}

fn reject_output_input_equality<'a>(
    output: Option<&PathBuf>,
    inputs: impl Iterator<Item = &'a PathBuf>,
    command: &str,
) -> Result<(), String> {
    if let Some(output) = output {
        let output = effective_output_path(output)?;
        for input in inputs {
            if effective_input_path(input)? == output {
                return Err(format!("{command} output must not overwrite an input path"));
            }
        }
    }
    Ok(())
}

fn reject_model_output_descendant(output: Option<&PathBuf>, model: &PathBuf) -> Result<(), String> {
    let Some(output) = output else {
        return Ok(());
    };
    let model = effective_input_path(model)?;
    let output = effective_output_path(output)?;
    if output.starts_with(&model) {
        return Err("run plan output must not be inside the model directory".into());
    }
    Ok(())
}

fn effective_input_path(path: &PathBuf) -> Result<PathBuf, String> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => absolute_lexical_path(path),
        Err(error) => Err(format!(
            "cannot resolve input path {}: {error}",
            path.display()
        )),
    }
}

fn effective_output_path(path: &PathBuf) -> Result<PathBuf, String> {
    match fs::canonicalize(path) {
        Ok(path) => return Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "cannot resolve output path {}: {error}",
                path.display()
            ));
        }
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("output path {} has no file name", path.display()))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let parent = match fs::canonicalize(parent) {
        Ok(parent) => parent,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            absolute_lexical_path(&parent.to_path_buf())?
        }
        Err(error) => {
            return Err(format!(
                "cannot resolve output parent {}: {error}",
                parent.display()
            ));
        }
    };
    Ok(parent.join(file_name))
}

fn absolute_lexical_path(path: &PathBuf) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.clone()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("cannot resolve current directory: {error}"))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            component => normalized.push(component.as_os_str()),
        }
    }
    Ok(normalized)
}

fn report_schema_errors(errors: &[impl std::fmt::Display]) -> ExitCode {
    for error in errors {
        eprintln!("error: {error}");
    }
    ExitCode::from(2)
}

fn report_host_error(error: &azimuth::adapter_host::HostError) {
    eprintln!("error: {error}");
    if !error.stderr.is_empty() {
        eprintln!("adapter stderr: {}", error.stderr);
    }
}

fn parse_run_options(args: &[String], inspect: bool) -> Result<RunOptions, String> {
    let mut bundles = Vec::new();
    let mut format = None;
    let mut out = None;
    let mut index = 0;
    while index < args.len() {
        let value = |name: &str| {
            args.get(index + 1)
                .cloned()
                .ok_or_else(|| format!("`{name}` needs a value"))
        };
        match args[index].as_str() {
            "--bundle" => {
                bundles.push(PathBuf::from(value("--bundle")?));
                index += 2;
            }
            "--format" if inspect => {
                if format.is_some() {
                    return Err("`--format` may be supplied only once".into());
                }
                format = Some(match value("--format")?.as_str() {
                    "text" => RunOutputFormat::Text,
                    "json" => RunOutputFormat::Json,
                    other => return Err(format!("unknown run inspection format `{other}`")),
                });
                index += 2;
            }
            "--out" if inspect => {
                if out.is_some() {
                    return Err("`--out` may be supplied only once".into());
                }
                out = Some(PathBuf::from(value("--out")?));
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown run option `{value}`"));
            }
            value => return Err(format!("unexpected run positional argument `{value}`")),
        }
    }
    if bundles.is_empty() {
        return Err("run verification needs at least one `--bundle <file>`".into());
    }
    reject_output_input_equality(out.as_ref(), bundles.iter(), "run inspection")?;
    Ok(RunOptions {
        bundles,
        format: format.unwrap_or(RunOutputFormat::Text),
        out,
    })
}

fn load_run_bundles(paths: &[PathBuf]) -> Result<Vec<azimuth::run::RunBundle>, ExitCode> {
    let mut bundles = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match azimuth::run::load(path) {
            Ok(bundle) => bundles.push(bundle),
            Err(mut found) => errors.append(&mut found),
        }
    }
    if errors.is_empty() {
        Ok(bundles)
    } else {
        for error in &errors {
            eprintln!("error: {error}");
        }
        eprintln!(
            "\n{} Run bundle schema error(s); no account was derived",
            errors.len()
        );
        Err(ExitCode::from(2))
    }
}

fn command_run_verify(options: RunOptions) -> Result<ExitCode, String> {
    let bundles = match load_run_bundles(&options.bundles) {
        Ok(bundles) => bundles,
        Err(code) => return Ok(code),
    };
    let findings = azimuth::run::verify_set(&bundles);
    for finding in &findings {
        println!("{}: {}: {}", finding.run_id, finding.code, finding.detail);
    }
    let unique = unique_run_bundles(&bundles);
    let runs = unique
        .iter()
        .map(|bundle| bundle.run_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    if !findings.is_empty() {
        println!();
    }
    println!("{} bundle revision(s) across {runs} Run(s)", unique.len());
    if findings.is_empty() {
        println!("protocol-consistent");
    } else {
        println!("{} protocol finding(s)", findings.len());
    }
    println!("current model: unresolved");
    println!("Assurance State: unresolved");
    Ok(if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn command_run_inspect(options: RunOptions) -> Result<ExitCode, String> {
    let bundles = match load_run_bundles(&options.bundles) {
        Ok(bundles) => bundles,
        Err(code) => return Ok(code),
    };
    let findings = azimuth::run::verify_set(&bundles);
    let rendered = match options.format {
        RunOutputFormat::Text => run_inspection_text(&bundles, &findings),
        RunOutputFormat::Json => run_inspection_json(&bundles, &findings).to_string_pretty(),
    };
    publish_output(rendered.as_bytes(), options.out.as_ref())?;
    Ok(if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn publish_output(bytes: &[u8], output: Option<&PathBuf>) -> Result<(), String> {
    let Some(output) = output else {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        lock.write_all(bytes)
            .and_then(|_| lock.flush())
            .map_err(|error| format!("cannot write standard output: {error}"))?;
        return Ok(());
    };

    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let file_name = output
        .file_name()
        .ok_or_else(|| format!("output path {} has no file name", output.display()))?
        .to_string_lossy();
    let mut last_collision = None;
    for _ in 0..128 {
        let sequence = OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{file_name}.azimuth-{}-{sequence}.tmp",
            std::process::id()
        ));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                last_collision = Some(error);
                continue;
            }
            Err(error) => {
                return Err(format!(
                    "cannot create temporary output beside {}: {error}",
                    output.display()
                ));
            }
        };
        if let Err(error) = file.write_all(bytes).and_then(|_| file.flush()) {
            drop(file);
            let _ = fs::remove_file(&temporary);
            return Err(format!("cannot write {}: {error}", output.display()));
        }
        drop(file);
        if let Err(error) = fs::rename(&temporary, output) {
            let _ = fs::remove_file(&temporary);
            return Err(format!(
                "cannot publish {} atomically: {error}",
                output.display()
            ));
        }
        return Ok(());
    }
    Err(format!(
        "cannot reserve a unique temporary output beside {}: {}",
        output.display(),
        last_collision
            .map(|error| error.to_string())
            .unwrap_or_else(|| "name collision".into())
    ))
}

fn unique_run_bundles(bundles: &[azimuth::run::RunBundle]) -> Vec<&azimuth::run::RunBundle> {
    let mut unique = Vec::new();
    for bundle in bundles {
        if !unique.contains(&bundle) {
            unique.push(bundle);
        }
    }
    unique.sort_by(|left, right| {
        left.run_id
            .cmp(&right.run_id)
            .then_with(|| left.bundle_revision.cmp(&right.bundle_revision))
            .then_with(|| left.bundle_fingerprint.cmp(&right.bundle_fingerprint))
    });
    unique
}

fn subject_kind(subject: &azimuth::run::Subject) -> &'static str {
    match subject {
        azimuth::run::Subject::Workspace { .. } => "workspace",
        azimuth::run::Subject::CiCandidate { .. } => "ci-candidate",
        azimuth::run::Subject::Artifact { .. } => "artifact",
        azimuth::run::Subject::Deployment { .. } => "deployment",
        azimuth::run::Subject::Service { .. } => "service",
        azimuth::run::Subject::MonitoringWindow { .. } => "monitoring-window",
    }
}

fn run_inspection_text(
    bundles: &[azimuth::run::RunBundle],
    findings: &[azimuth::run::Finding],
) -> String {
    let unique = unique_run_bundles(bundles);
    let mut rendered = String::new();
    let _ = writeln!(rendered, "Run bundle inspection");
    let _ = writeln!(
        rendered,
        "Protocol: {}",
        if findings.is_empty() {
            "consistent".into()
        } else {
            format!("{} finding(s)", findings.len())
        }
    );
    let _ = writeln!(rendered, "Current model: unresolved");
    let _ = writeln!(rendered, "Assurance State: unresolved");
    for bundle in unique {
        let _ = writeln!(rendered);
        let _ = writeln!(rendered, "Run {}", bundle.run_id);
        let _ = writeln!(
            rendered,
            "  Bundle: revision {} {}",
            bundle.bundle_revision, bundle.bundle_fingerprint
        );
        let _ = writeln!(
            rendered,
            "  Subject: {} {}",
            subject_kind(&bundle.subject),
            bundle.subject_fingerprint
        );
        let _ = writeln!(rendered, "  Status: {}", bundle.status.name());
        let mut observations = bundle.check_executions.iter().collect::<Vec<_>>();
        observations.sort_by(|left, right| {
            left.check
                .id
                .cmp(&right.check.id)
                .then_with(|| left.check.fingerprint.cmp(&right.check.fingerprint))
        });
        for execution in observations {
            let _ = writeln!(
                rendered,
                "  Observation: {} {} {}",
                execution.check.id,
                execution.observation.outcome.name(),
                execution.observation.fingerprint
            );
        }
        let mut challenges = bundle.challenger_executions.iter().collect::<Vec<_>>();
        challenges.sort_by(|left, right| left.challenge.cmp(&right.challenge));
        for execution in challenges {
            let _ = writeln!(
                rendered,
                "  Challenge Result: {} {} {} {}",
                execution.challenge,
                execution.target.fingerprint,
                execution.result.outcome.name(),
                execution.result.fingerprint
            );
        }
    }
    for finding in findings {
        let _ = writeln!(rendered);
        let _ = writeln!(
            rendered,
            "Finding: {} {} {}",
            finding.run_id, finding.code, finding.detail
        );
    }
    rendered
}

fn run_inspection_json(
    bundles: &[azimuth::run::RunBundle],
    findings: &[azimuth::run::Finding],
) -> azimuth::json::Json {
    use azimuth::json::Json;
    let bundles = unique_run_bundles(bundles)
        .into_iter()
        .map(|bundle| {
            let mut observations = bundle.check_executions.iter().collect::<Vec<_>>();
            observations.sort_by(|left, right| {
                left.check
                    .id
                    .cmp(&right.check.id)
                    .then_with(|| left.check.fingerprint.cmp(&right.check.fingerprint))
            });
            let observations = observations
                .into_iter()
                .map(|execution| {
                    Json::obj(vec![
                        ("check", Json::str(&execution.check.id)),
                        ("check_fingerprint", Json::str(&execution.check.fingerprint)),
                        ("outcome", Json::str(execution.observation.outcome.name())),
                        ("fingerprint", Json::str(&execution.observation.fingerprint)),
                    ])
                })
                .collect();
            let mut challenges = bundle.challenger_executions.iter().collect::<Vec<_>>();
            challenges.sort_by(|left, right| left.challenge.cmp(&right.challenge));
            let challenges = challenges
                .into_iter()
                .map(|execution| {
                    Json::obj(vec![
                        ("challenge", Json::str(&execution.challenge)),
                        ("challenger", Json::str(&execution.challenger.id)),
                        (
                            "challenger_fingerprint",
                            Json::str(&execution.challenger.fingerprint),
                        ),
                        ("target_kind", Json::str(execution.target.kind.name())),
                        ("target", Json::str(&execution.target.id)),
                        (
                            "target_fingerprint",
                            Json::str(&execution.target.fingerprint),
                        ),
                        ("outcome", Json::str(execution.result.outcome.name())),
                        ("fingerprint", Json::str(&execution.result.fingerprint)),
                    ])
                })
                .collect();
            Json::obj(vec![
                ("run_id", Json::str(&bundle.run_id)),
                ("bundle_revision", Json::Num(bundle.bundle_revision as f64)),
                ("bundle_fingerprint", Json::str(&bundle.bundle_fingerprint)),
                (
                    "corrects",
                    bundle
                        .corrects
                        .as_ref()
                        .map(Json::str)
                        .unwrap_or(Json::Null),
                ),
                ("subject_kind", Json::str(subject_kind(&bundle.subject))),
                (
                    "subject_fingerprint",
                    Json::str(&bundle.subject_fingerprint),
                ),
                ("status", Json::str(bundle.status.name())),
                ("observations", Json::Arr(observations)),
                ("challenge_results", Json::Arr(challenges)),
            ])
        })
        .collect();
    let findings: Vec<Json> = findings
        .iter()
        .map(|finding| {
            Json::obj(vec![
                ("run_id", Json::str(&finding.run_id)),
                ("code", Json::str(&finding.code)),
                ("detail", Json::str(&finding.detail)),
            ])
        })
        .collect();
    Json::obj(vec![
        ("format", Json::str("azimuth-run-inspection")),
        ("version", Json::Num(1.0)),
        ("protocol_consistent", Json::Bool(findings.is_empty())),
        ("model_authority", Json::str("unresolved")),
        ("assurance_state", Json::str("unresolved")),
        ("bundles", Json::Arr(bundles)),
        ("findings", Json::Arr(findings)),
    ])
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        model: PathBuf::from("azimuth/model"),
        standards: PathBuf::from("azimuth/standards/verification.md"),
        workspace: PathBuf::new(),
        manifests: Vec::new(),
        only: Vec::new(),
        out: None,
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
            other => return Err(format!("unexpected positional argument `{other}`")),
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

fn report_findings(model: &azimuth::model::Model, findings: &[validation::Finding]) {
    for finding in findings {
        let where_ = if finding.line > 0 {
            format!("{}:{}", finding.path, finding.line)
        } else {
            finding.path.clone()
        };
        let claim = finding.claim.clone().unwrap_or_else(|| "-".into());
        let level = finding
            .criticality
            .map(|criticality| format!(" ({})", criticality.name()))
            .unwrap_or_default();
        println!(
            "{where_}: {} {} {} {claim}{level}\n    {}\n    help: {}",
            finding.severity.name(),
            finding.kind.category().name(),
            finding.kind.name(),
            finding.detail,
            finding.kind.help()
        );
    }
    let summary = validation::summarize(model, findings);
    let by_kind = validation::counts_by_kind(findings)
        .into_iter()
        .map(|(kind, count)| format!("{count} {kind}"))
        .collect::<Vec<_>>();
    println!();
    println!("{} claims in {} spec(s)", summary.claims, model.specs.len());
    if by_kind.is_empty() {
        println!("no findings");
    } else {
        println!("{}", by_kind.join(" · "));
    }
    println!(
        "{} error(s), {} warning(s)",
        summary.errors, summary.warnings
    );
}

fn command_validate(options: Options) -> Result<ExitCode, String> {
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

    let findings = validation::validate(&loaded.model);
    let summary = validation::summarize(&loaded.model, &findings);
    report_findings(&loaded.model, &findings);

    Ok(if summary.errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn command_report(args: &[String]) -> Result<ExitCode, String> {
    let Some(operation) = args.first() else {
        return Err("report needs traceability".into());
    };
    if operation != "traceability" {
        return Err(format!("unknown report `{operation}`"));
    }
    let options = parse_options(&args[1..])?;
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
    let json = azimuth::traceability::project(&loaded.model)
        .to_json()
        .to_string_pretty();
    match options.out {
        Some(path) => std::fs::write(&path, json)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?,
        None => print!("{json}"),
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

    let findings = validation::validate(&loaded.model);
    let json = loaded.model.to_json(&findings).to_string_pretty();

    match options.out {
        Some(path) => std::fs::write(&path, json)
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?,
        None => print!("{json}"),
    }
    Ok(ExitCode::SUCCESS)
}
