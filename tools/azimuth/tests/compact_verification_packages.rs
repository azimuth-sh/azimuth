//! Compact package contracts use synthetic operations and independent producer schemas.
use azimuth::account_verification::parse_account_verification;
use azimuth::json;
use azimuth::verification_packages::{validate_accounts, Producer, SURFACE};
use azimuth::workspace::{Area, Workspace};

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DOCUMENT: &str = "# Verification: catalog\n\n## Shared inputs\n\n### Surface: operations\n- Fields:\n  - `Key`\n  - `Transport`\n- Area: `api`\n\nDiscover actual registrations.\n\n## Claim verification: restricted-access\n\n### Case verification: rejected-access\n\n#### Check: rejects-access\n- Surface: `operations`\n- Select:\n  - Member.Transport: `http`\n- Evidence bindings:\n  - Contribution: Establishes rejection for selected operations.\n\nExercise rejected requests and a successful control.\n";

fn workspace() -> Workspace {
    Workspace {
        packages: vec![SURFACE.into()],
        areas: vec![Area {
            id: "api".into(),
            mounts: vec![],
        }],
        ..Workspace::default()
    }
}
fn producer() -> Producer {
    Producer { package: SURFACE.into(), contract: "surface-enumerator".into(), entity: "operations".into(), site: "Catalog::Operations".into(), file: "src/catalog.rs".into(), lang: "rust".into(), source_fingerprint: SHA.into(), source: None, inputs: vec![], schema: Some(json::parse(r#"{"kind":"record","fields":[{"name":"Key","schema":{"kind":"string"}},{"name":"Transport","schema":{"kind":"enum","values":["http","socket"]}}]}"#).unwrap()) }
}
fn errors(document: &str, workspace: &Workspace, producers: &[Producer]) -> String {
    let account = parse_account_verification("verification.md", document).unwrap();
    validate_accounts(&[account], workspace, producers)
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
#[test]
fn case_inheritance_retains_check_case_binding_without_an_authored_binding_id() {
    let account = parse_account_verification("verification.md", DOCUMENT).unwrap();
    assert_eq!(account.checks[0].definition.id, "rejects-access");
    assert_eq!(account.checks[0].bindings[0].case, "rejected-access");
    assert_eq!(account.checks[0].selectors["Member.Transport"], "`http`");
    assert!(errors(DOCUMENT, &workspace(), &[producer()]).is_empty());
}
#[test]
fn source_enum_and_published_fields_both_constrain_selectors() {
    let wrong_value = DOCUMENT.replace("Member.Transport: `http`", "Member.Transport: `invented`");
    assert!(errors(&wrong_value, &workspace(), &[producer()])
        .contains("selector value/type is unsupported"));
    let unpublished = DOCUMENT.replace("Member.Transport:", "Member.Hidden:");
    assert!(errors(&unpublished, &workspace(), &[producer()]).contains("unknown selector field"));
    let missing_output = DOCUMENT.replace("  - `Transport`", "  - `Missing`");
    assert!(errors(&missing_output, &workspace(), &[producer()])
        .contains("published field `Missing` missing"));
}
#[test]
fn package_declarations_require_explicit_enablement() {
    let mut disabled = workspace();
    disabled.packages.clear();
    assert!(errors(DOCUMENT, &disabled, &[producer()]).contains("is not enabled"));
}
#[test]
fn inherited_contribution_requires_an_enclosing_case() {
    let outside = DOCUMENT
        .replace("### Case verification: rejected-access\n\n", "")
        .replace("#### Check:", "### Check:");
    assert!(parse_account_verification("verification.md", &outside).is_err());
}
