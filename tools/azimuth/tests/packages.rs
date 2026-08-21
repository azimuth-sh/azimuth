//! Strict source-linkage manifest tests. Synthetic fixtures only.

use azimuth::json;
use azimuth::manifest;

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn parse(source: &str) -> Result<manifest::Manifest, String> {
    let root = json::parse(source).unwrap();
    manifest::parse("manifest.json", &root).map_err(|errors| {
        errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    })
}

#[test]
fn parses_stable_check_implementation_linkage() {
    let source = format!(
        r#"{{"check_implementations":[{{
          "check":"payments/recovery-under-loss",
          "site":"recovery::replay_after_loss",
          "file":"src/recovery.rs",
          "lang":"rust",
          "source_fingerprint":"{SHA}",
          "area":"payments",
          "address_kind":"rust-item",
          "address":"recovery::replay_after_loss",
          "mount":"code"
        }}]}}"#
    );
    let manifest = parse(&source).unwrap();
    let implementation = &manifest.check_implementations[0];
    assert_eq!(implementation.check, "payments/recovery-under-loss");
    assert_eq!(
        implementation.semantic_identity(),
        "payments|rust-item|recovery::replay_after_loss"
    );
    assert_eq!(implementation.source_fingerprint, SHA);
}

#[test]
fn rejects_every_alpha_one_evidence_collection() {
    for key in ["covers", "mechanism_covers", "observations"] {
        let error = parse(&format!(r#"{{"{key}":[]}}"#)).unwrap_err();
        assert!(error.contains("legacy manifest key"), "{key}: {error}");
    }
}

#[test]
fn rejects_unknown_and_missing_check_implementation_fields() {
    let error = parse(&format!(
        r#"{{"check_implementations":[{{
          "check":"payments/recovery-under-loss",
          "site":"recovery::replay_after_loss",
          "file":"src/recovery.rs",
          "lang":"rust",
          "source_fingerprint":"{SHA}",
          "claim":"payments/recovery#accepted-write-replayed"
        }}]}}"#
    ))
    .unwrap_err();
    assert!(error.contains("unknown field `claim`"), "{error}");

    let missing = parse(
        r#"{"check_implementations":[{
          "check":"payments/recovery-under-loss",
          "site":"recovery::replay_after_loss",
          "file":"src/recovery.rs",
          "lang":"rust"
        }]}"#,
    )
    .unwrap_err();
    assert!(missing.contains("source_fingerprint"), "{missing}");
}

#[test]
fn rejects_invalid_fingerprints_partial_identity_and_duplicates() {
    let invalid = parse(
        r#"{"check_implementations":[{
          "check":"payments/recovery-under-loss",
          "site":"recovery::replay_after_loss",
          "file":"src/recovery.rs",
          "lang":"rust",
          "source_fingerprint":"abc",
          "area":"payments"
        }]}"#,
    )
    .unwrap_err();
    assert!(invalid.contains("SHA-256"), "{invalid}");
    assert!(invalid.contains("partial source identity"), "{invalid}");

    let implementation = format!(
        r#"{{"check":"payments/recovery-under-loss","site":"recovery::replay_after_loss",
        "file":"src/recovery.rs","lang":"rust","source_fingerprint":"{SHA}"}}"#
    );
    let duplicate = parse(&format!(
        "{{\"check_implementations\":[{implementation},{implementation}]}}"
    ))
    .unwrap_err();
    assert!(
        duplicate.contains("duplicate Check implementation"),
        "{duplicate}"
    );
}

#[test]
fn accepts_all_six_current_collections_and_rejects_unknown_top_level_keys() {
    let source = format!(
        r#"{{
          "realizes":[{{"spec":"alpha","scenario":"works","site":"A","file":"a.rs","lang":"rust"}}],
          "check_implementations":[{{"check":"alpha/works","site":"T","file":"t.rs",
            "lang":"rust","source_fingerprint":"{SHA}"}}],
          "mechanism_implementations":[{{"spec":"alpha","mechanism":"guard",
            "binding":"rust:A","file":"a.rs","lang":"rust"}}],
          "class_members":[{{"class":"routes","site":"A","file":"a.rs","lang":"rust"}}],
          "enumerations":[{{"class":"routes","kind":"routes","source":"routes.json",
            "source_fingerprint":"{SHA}"}}],
          "artifacts":[{{"id":"schema:users","kind":"unique-index","file":"schema.sql"}}]
        }}"#
    );
    let parsed = parse(&source).unwrap();
    assert_eq!(parsed.realizes.len(), 1);
    assert_eq!(parsed.check_implementations.len(), 1);
    assert_eq!(parsed.mechanism_implementations.len(), 1);
    assert_eq!(parsed.class_members.len(), 1);
    assert_eq!(parsed.enumerations.len(), 1);
    assert_eq!(parsed.artifacts.len(), 1);

    let error = parse(r#"{"realizes":[],"extra":[]}"#).unwrap_err();
    assert!(error.contains("unknown manifest key `extra`"), "{error}");
}
