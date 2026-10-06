//! Supplied source identities remain stable through ownership-only assembly.
use azimuth::load;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "azimuth-qualified-source-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("model/catalog")).unwrap();
        fs::write(root.join("model/catalog/spec.md"), "# Spec: catalog\n\n## Claim: list-is-returned\nCriticality: routine\n\nThe catalog SHALL return its list.\n\n### Case: list-requested\nEvent: list is requested\nRequired: list is returned\n").unwrap();
        fs::write(root.join("manifest.json"), format!(r#"{{"realizes":[{{"claim":"list-is-returned","site":"Catalog::List","file":"relocated/list.rs","lang":"rust","source_fingerprint":"{SHA}","area":"api","address_kind":"rust-item","address":"Catalog::List","mount":"implementation"}}]}}"#)).unwrap();
        Self(root)
    }
    fn workspace(&self, areas: &str) {
        fs::write(self.0.join("workspace.json"), format!(r#"{{"format":"azimuth-workspace","version":1,"areas":{areas},"surfaces":[],"realization_obligations":[]}}"#)).unwrap();
    }
    fn load(&self) -> Result<azimuth::Loaded, Vec<azimuth::diag::Diag>> {
        load(
            &self.0.join("model"),
            &self.0.join("standards.md"),
            &self.0.join("workspace.json"),
            &[self.0.join("manifest.json")],
            &[],
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn ownership_only_area_preserves_supplied_semantic_address_and_mount() {
    let fixture = Fixture::new();
    fixture.workspace(r#"[{"id":"api","mounts":[]}]"#);
    let loaded = fixture.load().unwrap();
    let source = loaded.model.realizes[0].source.as_ref().unwrap();
    assert_eq!(source.area, "api");
    assert_eq!(source.mount, "implementation");
    assert_eq!(source.address, "Catalog::List");
    assert_eq!(source.kind, "rust-item");
}
#[test]
fn qualified_sources_cannot_name_an_undeclared_area_or_mount() {
    let fixture = Fixture::new();
    for (areas, expected) in [
        (r#"[{"id":"other","mounts":[]}]"#, "unknown Area"),
        (
            r#"[{"id":"api","mounts":[{"id":"source","path":"src"}]}]"#,
            "unknown mount",
        ),
    ] {
        fixture.workspace(areas);
        let errors = fixture
            .load()
            .unwrap_err()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(errors.contains(expected), "{errors}");
    }
}
