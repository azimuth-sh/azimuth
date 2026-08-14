//! Model-package discovery tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TemporaryModel {
    root: PathBuf,
}

impl TemporaryModel {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "azimuth-packages-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn write(&self, relative: &str, source: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }

    fn model(&self) -> PathBuf {
        self.root.join("model")
    }

    fn missing_standards(&self) -> PathBuf {
        self.root.join("standards/verification.md")
    }
}

impl Drop for TemporaryModel {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

const SPEC: &str = "\
# Spec: alpha/beta

## Requirement: thing-holds
Criticality: routine

The system SHALL hold the thing.

### Scenario: thing-held
WHEN the thing is examined
THEN it is held
";

fn load(root: &Path, standards: &Path) -> azimuth::Loaded {
    azimuth::load(root, standards, &root.join("../workspace.json"), &[], &[])
        .expect("model packages load")
}

#[test]
fn a_routine_package_needs_only_a_spec() {
    let fixture = TemporaryModel::new();
    fixture.write("model/alpha/beta/spec.md", SPEC);
    fixture.write("model/alpha/beta/notes.md", "# Not an Azimuth artifact");

    let loaded = load(&fixture.model(), &fixture.missing_standards());

    assert_eq!(loaded.model.specs.len(), 1);
    assert!(loaded.model.designs.is_empty());
    assert!(loaded.model.plans.is_empty());
    assert!(loaded.model.judgments.is_empty());
}

#[test]
fn sibling_facets_form_one_package() {
    let fixture = TemporaryModel::new();
    fixture.write("model/alpha/beta/spec.md", SPEC);
    fixture.write("model/alpha/beta/design.md", "# Design: alpha/beta\n");
    fixture.write(
        "model/alpha/beta/verification.md",
        "# Verification: alpha/beta\n",
    );
    fixture.write("model/alpha/beta/judgments.md", "# Judgments: alpha/beta\n");

    let loaded = load(&fixture.model(), &fixture.missing_standards());

    assert_eq!(loaded.model.designs.len(), 1);
    assert_eq!(loaded.model.plans.len(), 1);
    assert_eq!(loaded.model.judgments.len(), 1);
    assert!(!loaded
        .warnings
        .iter()
        .any(|warning| warning.to_string().contains("not beside")));
}

#[test]
fn declared_identity_wins_over_package_location() {
    let fixture = TemporaryModel::new();
    fixture.write("model/wrong/location/spec.md", SPEC);

    let loaded = load(&fixture.model(), &fixture.missing_standards());

    assert_eq!(loaded.model.specs[0].id, "alpha/beta");
    assert!(loaded
        .warnings
        .iter()
        .any(|warning| warning.to_string().contains("does not match its location")));
}

#[test]
fn a_non_sibling_facet_is_visible_as_a_navigation_warning() {
    let fixture = TemporaryModel::new();
    fixture.write("model/alpha/beta/spec.md", SPEC);
    fixture.write("model/misplaced/design.md", "# Design: alpha/beta\n");

    let loaded = load(&fixture.model(), &fixture.missing_standards());

    assert_eq!(loaded.model.designs[0].spec, "alpha/beta");
    assert!(loaded.warnings.iter().any(|warning| warning
        .to_string()
        .contains("design for `alpha/beta` is not beside")));
}
