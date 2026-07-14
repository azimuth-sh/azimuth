//! The traceability matrix, derived from a spec's scenarios and the linkage tags on code and
//! tests. Pure core (`build`) plus a spec parser and a source scanner, all std-only so the tool
//! builds offline. The port of the drim-dev C# machine tier, made language-agnostic: the spec is
//! markdown and the tags are a line-comment convention any language can carry.

use std::fmt;

pub mod manifest;
pub mod scan;
pub mod spec;

/// How much of the real system a check runs against — the radius of blast. A ladder: a check at a
/// higher scope also satisfies a lower requirement. `integration` folds into `component` (a
/// component test *is* the service-integration level here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Unit,
    Component,
    E2e,
}

impl Scope {
    pub fn parse(text: &str) -> Option<Scope> {
        match text.trim().to_ascii_lowercase().as_str() {
            "unit" => Some(Scope::Unit),
            "component" | "integration" => Some(Scope::Component),
            "e2e" => Some(Scope::E2e),
            _ => None,
        }
    }
}

/// The logical form of the claim: `example` (∃ one case) or `invariant` (∀ a property over all
/// inputs/states). A ladder: an invariant also satisfies an example requirement. `completeness`
/// (no implemented case is lost) is a named invariant, so it folds in here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Quantification {
    Example,
    Invariant,
}

impl Quantification {
    pub fn parse(text: &str) -> Option<Quantification> {
        match text.trim().to_ascii_lowercase().as_str() {
            "example" => Some(Quantification::Example),
            "invariant" | "completeness" => Some(Quantification::Invariant),
            _ => None,
        }
    }
}

/// Where the expected result came from. A descriptive label, never gated: it records *how* the
/// oracle was obtained, not the *strength* of the proof (strength is the (scope, quantification)
/// pair). Kept for the code-map and for teaching; the matrix never reddens on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Oracle {
    Direct,
    Golden,
    Metamorphic,
    ModelBased,
    Contract,
}

impl Oracle {
    pub fn parse(text: &str) -> Option<Oracle> {
        match text.trim().to_ascii_lowercase().as_str() {
            "direct" => Some(Oracle::Direct),
            "golden" => Some(Oracle::Golden),
            "metamorphic" => Some(Oracle::Metamorphic),
            "model-based" => Some(Oracle::ModelBased),
            "contract" => Some(Oracle::Contract),
            _ => None,
        }
    }
}

impl fmt::Display for Oracle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Oracle::Direct => "direct",
            Oracle::Golden => "golden",
            Oracle::Metamorphic => "metamorphic",
            Oracle::ModelBased => "model-based",
            Oracle::Contract => "contract",
        };
        write!(f, "{text}")
    }
}

/// The honest kind of check a scenario demands: the pair of orthogonal axes that encode proof
/// *strength*. The matrix reddens (`WrongForm`) when neither axis is met.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Form {
    pub scope: Scope,
    pub quantification: Quantification,
}

impl Form {
    pub fn new(scope: Scope, quantification: Quantification) -> Form {
        Form {
            scope,
            quantification,
        }
    }

    /// A delivered form satisfies a required one when it is at least as strong on *both* axes — a
    /// higher scope or a stronger quantification still counts, an under-proof on either does not.
    pub fn satisfies(self, required: Form) -> bool {
        self.scope >= required.scope && self.quantification >= required.quantification
    }
}

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}",
            format!("{:?}", self.scope).to_ascii_lowercase(),
            format!("{:?}", self.quantification).to_ascii_lowercase(),
        )
    }
}

/// The (spec-id, req-id, scenario-id) triple — the stable key that survives display-name renames.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub spec_id: String,
    pub req_id: String,
    pub scenario_id: String,
}

/// A specified behavior: the coverage unit. `exposes`/`upholds` ride on the scenario (not on code
/// tags): a site *realizing* an `exposes` scenario joins that surface class, and a site realizing
/// an `upholds` scenario discharges that invariant there.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub key: Key,
    pub required_form: Form,
    pub name: String,
    pub exposes: Option<String>,
    pub upholds: Option<String>,
}

/// A named cross-cutting invariant: a guarantee that must hold across every site in a surface
/// class. Declared once in the owner spec (`## Invariant`). Neither requirement nor scenario — it
/// binds a class of surfaces to a guard the tool demands at each of them.
#[derive(Debug, Clone)]
pub struct Invariant {
    pub id: String,
    pub over: String,
    pub references: Vec<String>,
    pub name: String,
    pub spec_id: String,
}

/// A parsed spec's two authored artifacts: its scenarios and its invariant declarations.
#[derive(Debug, Clone, Default)]
pub struct ParsedSpec {
    pub scenarios: Vec<Scenario>,
    pub invariants: Vec<Invariant>,
}

/// A `covers` tag on a test: this test verifies that scenario, at this form. The `oracle` is a
/// descriptive note, never gated.
#[derive(Debug, Clone)]
pub struct Tag {
    pub key: Key,
    pub form: Form,
    pub oracle: Option<Oracle>,
    pub site: String,
}

/// A `realizes` tag on production code: this site is on that scenario's path. No form.
#[derive(Debug, Clone)]
pub struct Realization {
    pub key: Key,
    pub site: String,
}

/// A test the emitter reports as untraced: it lives under a traced root (an opt-in area) yet
/// declares no scenario and is not explicitly opted out. No key — it names no scenario; that
/// absence *is* the defect. The area scope (only tests under a declared root contribute) is applied
/// at emission, so the core reports every entry a manifest carries.
#[derive(Debug, Clone)]
pub struct UntracedTest {
    pub site: String,
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoleKind {
    Uncovered,
    Unrealized,
    WrongForm,
    Dangling,
    DanglingRealization,
    /// A site in an invariant's surface class that does not discharge the invariant — the leak the
    /// per-scenario matrix cannot see (the scenario is realized elsewhere, so it looks covered).
    InvariantBreach {
        invariant_id: String,
        site: String,
    },
    /// An invariant `over` a class with no exposure sites — nothing to guard (likely a typo).
    DanglingInvariant {
        invariant_id: String,
    },
    /// A scenario that `upholds` an invariant no spec declares.
    DanglingUpholds {
        invariant_id: String,
    },
    /// A test under a traced root (an opt-in area) that declares no scenario — the dual of an
    /// uncovered scenario. It may exercise behavior the spec never named, and without this it stays
    /// invisible.
    UntracedTest {
        site: String,
    },
}

/// A defect in the matrix. `key` names the scenario for scenario-scoped holes; invariant-scoped
/// holes carry their context in the `kind` instead and leave `key` empty.
#[derive(Debug, Clone)]
pub struct Hole {
    pub kind: HoleKind,
    pub key: Option<Key>,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub scenario: Scenario,
    pub realizations: Vec<Realization>,
    pub covering_tags: Vec<Tag>,
}

#[derive(Debug, Clone)]
pub struct Matrix {
    pub rows: Vec<Row>,
    pub holes: Vec<Hole>,
}

impl Matrix {
    pub fn is_whole(&self) -> bool {
        self.holes.is_empty()
    }
}

/// The pure core: scenarios + invariants + realizations (code) + tags (tests) → the matrix and its
/// holes. Two independent per-scenario axes (realized? covered?) plus a cross-cutting-invariant
/// pass, so structural holes and leaked guarantees both fall out without double-reporting.
// realizes: azimuth-rtm generate covered-and-realized-lists-both
// realizes: azimuth-rtm flag-uncovered second-scenario-uncovered
// realizes: azimuth-rtm flag-unrealized tested-but-unrealized
// realizes: azimuth-rtm flag-wrong-form under-proven-on-either-axis
// realizes: azimuth-rtm flag-dangling unknown-scenario-dangling
// realizes: azimuth-rtm flag-invariant-breach exposure-without-guard-breaches
// realizes: azimuth-rtm flag-dangling-invariant invariant-over-empty-class-dangles
// realizes: azimuth-rtm flag-dangling-upholds upholds-undeclared-invariant-dangles
// realizes: azimuth-rtm flag-untraced-test traced-test-without-scenario-untraced
pub fn build(
    scenarios: &[Scenario],
    invariants: &[Invariant],
    tags: &[Tag],
    realizations: &[Realization],
    untraced: &[UntracedTest],
) -> Matrix {
    let mut rows = Vec::new();
    let mut holes = Vec::new();

    for scenario in scenarios {
        let realizing: Vec<Realization> = realizations
            .iter()
            .filter(|realization| realization.key == scenario.key)
            .cloned()
            .collect();
        let covering: Vec<Tag> = tags
            .iter()
            .filter(|tag| tag.key == scenario.key)
            .cloned()
            .collect();

        if realizing.is_empty() {
            holes.push(Hole {
                kind: HoleKind::Unrealized,
                key: Some(scenario.key.clone()),
                detail: format!("'{}' has no code realizing it", scenario.name),
            });
        }

        if covering.is_empty() {
            holes.push(Hole {
                kind: HoleKind::Uncovered,
                key: Some(scenario.key.clone()),
                detail: format!("'{}' has no covering test", scenario.name),
            });
        } else if !covering
            .iter()
            .any(|tag| tag.form.satisfies(scenario.required_form))
        {
            holes.push(Hole {
                kind: HoleKind::WrongForm,
                key: Some(scenario.key.clone()),
                detail: format!(
                    "'{}' requires {} but is covered only by {}",
                    scenario.name,
                    scenario.required_form,
                    covering
                        .iter()
                        .map(|tag| tag.form.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }

        rows.push(Row {
            scenario: scenario.clone(),
            realizations: realizing,
            covering_tags: covering,
        });
    }

    let declared: std::collections::HashSet<&Key> =
        scenarios.iter().map(|scenario| &scenario.key).collect();

    for tag in tags {
        if !declared.contains(&tag.key) {
            holes.push(Hole {
                kind: HoleKind::Dangling,
                key: Some(tag.key.clone()),
                detail: format!(
                    "'{}' covers ({}, {}, {}) which no scenario declares",
                    tag.site, tag.key.spec_id, tag.key.req_id, tag.key.scenario_id
                ),
            });
        }
    }

    for realization in realizations {
        if !declared.contains(&realization.key) {
            holes.push(Hole {
                kind: HoleKind::DanglingRealization,
                key: Some(realization.key.clone()),
                detail: format!(
                    "'{}' realizes ({}, {}, {}) which no scenario declares",
                    realization.site,
                    realization.key.spec_id,
                    realization.key.req_id,
                    realization.key.scenario_id
                ),
            });
        }
    }

    check_invariants(scenarios, invariants, realizations, &mut holes);

    for test in untraced {
        holes.push(Hole {
            kind: HoleKind::UntracedTest {
                site: test.site.clone(),
            },
            key: None,
            detail: format!(
                "test '{}' ({}) is under a traced root but declares no scenario; add covers or [Untraced]",
                test.site, test.file
            ),
        });
    }

    Matrix { rows, holes }
}

/// The cross-cutting-invariant pass. A surface class is realization-defined: a site joins class `C`
/// by realizing a scenario that `exposes: C`, and discharges invariant `I` by realizing a scenario
/// that `upholds: I`. Every class member that does not discharge the invariant is a breach — the
/// new-surface leak the per-scenario matrix cannot catch, since the guard scenario is realized
/// somewhere else and the class looks covered.
fn check_invariants(
    scenarios: &[Scenario],
    invariants: &[Invariant],
    realizations: &[Realization],
    holes: &mut Vec<Hole>,
) {
    use std::collections::{BTreeSet, HashSet};

    let mut class_sites: std::collections::HashMap<&str, BTreeSet<String>> = Default::default();
    let mut upheld: std::collections::HashMap<&str, HashSet<String>> = Default::default();

    for scenario in scenarios {
        let sites: Vec<String> = realizations
            .iter()
            .filter(|realization| realization.key == scenario.key)
            .map(|realization| realization.site.clone())
            .collect();
        if let Some(class) = &scenario.exposes {
            class_sites
                .entry(class)
                .or_default()
                .extend(sites.iter().cloned());
        }
        if let Some(invariant) = &scenario.upholds {
            upheld
                .entry(invariant)
                .or_default()
                .extend(sites.iter().cloned());
        }
    }

    let declared: HashSet<&str> = invariants
        .iter()
        .map(|invariant| invariant.id.as_str())
        .collect();

    let mut reported_dangling: BTreeSet<&str> = BTreeSet::new();
    for scenario in scenarios {
        if let Some(invariant) = &scenario.upholds {
            if !declared.contains(invariant.as_str()) && reported_dangling.insert(invariant) {
                holes.push(Hole {
                    kind: HoleKind::DanglingUpholds {
                        invariant_id: invariant.clone(),
                    },
                    key: None,
                    detail: format!(
                        "'{}' upholds '{}' which no spec declares as an invariant",
                        scenario.name, invariant
                    ),
                });
            }
        }
    }

    for invariant in invariants {
        let sites = class_sites.get(invariant.over.as_str());
        let guarded = upheld.get(invariant.id.as_str());

        match sites {
            None => holes.push(Hole {
                kind: HoleKind::DanglingInvariant {
                    invariant_id: invariant.id.clone(),
                },
                key: None,
                detail: format!(
                    "invariant '{}' is over class '{}' which no exposure scenario realizes",
                    invariant.id, invariant.over
                ),
            }),
            Some(sites) => {
                for site in sites {
                    if !guarded.is_some_and(|guarded| guarded.contains(site)) {
                        holes.push(Hole {
                            kind: HoleKind::InvariantBreach {
                                invariant_id: invariant.id.clone(),
                                site: site.clone(),
                            },
                            key: None,
                            detail: format!(
                                "'{}' realizes an 'exposes: {}' scenario but discharges no 'upholds: {}' guard",
                                site, invariant.over, invariant.id
                            ),
                        });
                    }
                }
            }
        }
    }
}

/// The set of spec-ids in scope for a `--only` run: the requested specs plus the transitive
/// `references` closure of every invariant declared in an in-scope spec. Load-bearing — an
/// invariant's referenced capability must enter scope so its exposure sites join the class and a
/// leak there is visible; without it the leak surface is out of scope and invisible.
// realizes: azimuth-rtm scope-to-requested-specs references-closure-pulls-referenced
pub fn scope_closure(
    requested: &[String],
    invariants: &[Invariant],
) -> std::collections::HashSet<String> {
    let mut scope: std::collections::HashSet<String> = requested.iter().cloned().collect();
    loop {
        let mut grew = false;
        for invariant in invariants {
            if scope.contains(&invariant.spec_id) {
                for reference in &invariant.references {
                    grew |= scope.insert(reference.clone());
                }
            }
        }
        if !grew {
            break;
        }
    }
    scope
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(scenario_id: &str) -> Key {
        Key {
            spec_id: "spec".into(),
            req_id: "req".into(),
            scenario_id: scenario_id.into(),
        }
    }

    fn scenario(scenario_id: &str, form: Form) -> Scenario {
        Scenario {
            key: key(scenario_id),
            required_form: form,
            name: scenario_id.into(),
            exposes: None,
            upholds: None,
        }
    }

    fn tag(scenario_id: &str, form: Form) -> Tag {
        Tag {
            key: key(scenario_id),
            form,
            oracle: None,
            site: "T".into(),
        }
    }

    fn realization(scenario_id: &str) -> Realization {
        realization_at(scenario_id, "C")
    }

    fn realization_at(scenario_id: &str, site: &str) -> Realization {
        Realization {
            key: key(scenario_id),
            site: site.into(),
        }
    }

    fn unit_example() -> Form {
        Form::new(Scope::Unit, Quantification::Example)
    }

    fn component_invariant() -> Form {
        Form::new(Scope::Component, Quantification::Invariant)
    }

    #[test]
    // covers: azimuth-rtm generate covered-and-realized-lists-both unit example
    fn a_covered_and_realized_scenario_leaves_no_hole() {
        let matrix = build(
            &[scenario("runs", unit_example())],
            &[],
            &[tag("runs", unit_example())],
            &[realization("runs")],
            &[],
        );
        assert!(matrix.is_whole());
        assert_eq!(matrix.rows[0].covering_tags.len(), 1);
        assert_eq!(matrix.rows[0].realizations.len(), 1);
    }

    #[test]
    // covers: azimuth-rtm flag-uncovered second-scenario-uncovered unit example
    fn an_uncovered_scenario_is_flagged() {
        let matrix = build(
            &[scenario("runs", unit_example())],
            &[],
            &[],
            &[realization("runs")],
            &[],
        );
        assert_eq!(matrix.holes.len(), 1);
        assert_eq!(matrix.holes[0].kind, HoleKind::Uncovered);
    }

    #[test]
    // covers: azimuth-rtm flag-unrealized tested-but-unrealized unit example
    fn an_unrealized_scenario_is_flagged() {
        let matrix = build(
            &[scenario("runs", unit_example())],
            &[],
            &[tag("runs", unit_example())],
            &[],
            &[],
        );
        assert_eq!(matrix.holes.len(), 1);
        assert_eq!(matrix.holes[0].kind, HoleKind::Unrealized);
    }

    #[test]
    // covers: azimuth-rtm flag-wrong-form under-proven-on-either-axis unit example
    fn a_scenario_under_proven_on_either_axis_is_flagged() {
        let matrix = build(
            &[scenario("guarded", component_invariant())],
            &[],
            &[tag("guarded", unit_example())],
            &[realization("guarded")],
            &[],
        );
        assert_eq!(matrix.holes.len(), 1);
        assert_eq!(matrix.holes[0].kind, HoleKind::WrongForm);
    }

    #[test]
    fn a_stronger_form_than_required_still_satisfies() {
        let matrix = build(
            &[scenario("guarded", unit_example())],
            &[],
            &[tag("guarded", component_invariant())],
            &[realization("guarded")],
            &[],
        );
        assert!(matrix.is_whole());
    }

    #[test]
    fn a_scenario_under_proven_on_only_scope_is_flagged() {
        let matrix = build(
            &[scenario(
                "guarded",
                Form::new(Scope::E2e, Quantification::Example),
            )],
            &[],
            &[tag("guarded", unit_example())],
            &[realization("guarded")],
            &[],
        );
        assert_eq!(matrix.holes[0].kind, HoleKind::WrongForm);
    }

    #[test]
    // covers: azimuth-rtm flag-dangling unknown-scenario-dangling unit example
    fn a_tag_for_an_unknown_scenario_is_dangling() {
        let matrix = build(
            &[scenario("runs", unit_example())],
            &[],
            &[tag("runs", unit_example()), tag("ghost", unit_example())],
            &[realization("runs")],
            &[],
        );
        assert_eq!(matrix.holes.len(), 1);
        assert_eq!(matrix.holes[0].kind, HoleKind::Dangling);
    }

    fn exposure(scenario_id: &str, class: &str) -> Scenario {
        Scenario {
            exposes: Some(class.into()),
            ..scenario(scenario_id, unit_example())
        }
    }

    fn guard(scenario_id: &str, invariant: &str) -> Scenario {
        Scenario {
            upholds: Some(invariant.into()),
            ..scenario(scenario_id, component_invariant())
        }
    }

    fn invariant(id: &str, over: &str, references: &[&str]) -> Invariant {
        Invariant {
            id: id.into(),
            over: over.into(),
            references: references.iter().map(|r| r.to_string()).collect(),
            name: id.into(),
            spec_id: "spec".into(),
        }
    }

    #[test]
    fn a_class_site_that_discharges_the_invariant_leaves_no_breach() {
        // The public-detail site realizes both the exposure and the guard scenario.
        let matrix = build(
            &[
                exposure("detail-valid", "public-cert"),
                guard("detail-revoked-void", "revoked-hidden"),
            ],
            &[invariant("revoked-hidden", "public-cert", &[])],
            &[
                tag("detail-valid", unit_example()),
                tag("detail-revoked-void", component_invariant()),
            ],
            &[
                realization_at("detail-valid", "GetPublicCertificate"),
                realization_at("detail-revoked-void", "GetPublicCertificate"),
            ],
            &[],
        );
        assert!(matrix.is_whole(), "unexpected holes: {:?}", matrix.holes);
    }

    #[test]
    // covers: azimuth-rtm flag-invariant-breach exposure-without-guard-breaches unit example
    fn a_class_site_without_a_guard_is_an_invariant_breach() {
        // A new surface (the sitemap) realizes the exposure but discharges no guard → the leak.
        let matrix = build(
            &[
                exposure("detail-valid", "public-cert"),
                guard("detail-revoked-void", "revoked-hidden"),
                exposure("sitemap-lists-public", "public-cert"),
            ],
            &[invariant("revoked-hidden", "public-cert", &[])],
            &[
                tag("detail-valid", unit_example()),
                tag("detail-revoked-void", component_invariant()),
                tag("sitemap-lists-public", unit_example()),
            ],
            &[
                realization_at("detail-valid", "GetPublicCertificate"),
                realization_at("detail-revoked-void", "GetPublicCertificate"),
                realization_at("sitemap-lists-public", "GetSitemap"),
            ],
            &[],
        );
        let breaches: Vec<&Hole> = matrix
            .holes
            .iter()
            .filter(|hole| {
                matches!(&hole.kind, HoleKind::InvariantBreach { site, .. } if site == "GetSitemap")
            })
            .collect();
        assert_eq!(breaches.len(), 1, "holes: {:?}", matrix.holes);
    }

    #[test]
    // covers: azimuth-rtm flag-dangling-invariant invariant-over-empty-class-dangles unit example
    fn an_invariant_over_an_empty_class_is_dangling() {
        let matrix = build(
            &[],
            &[invariant("revoked-hidden", "public-cert", &[])],
            &[],
            &[],
            &[],
        );
        assert!(matches!(
            matrix.holes.as_slice(),
            [Hole {
                kind: HoleKind::DanglingInvariant { .. },
                ..
            }]
        ));
    }

    #[test]
    // covers: azimuth-rtm flag-dangling-upholds upholds-undeclared-invariant-dangles unit example
    fn a_scenario_upholding_an_undeclared_invariant_is_dangling() {
        let matrix = build(
            &[guard("guards-a-ghost", "no-such-invariant")],
            &[],
            &[tag("guards-a-ghost", component_invariant())],
            &[realization("guards-a-ghost")],
            &[],
        );
        assert!(matrix.holes.iter().any(|hole| matches!(
            &hole.kind,
            HoleKind::DanglingUpholds { invariant_id } if invariant_id == "no-such-invariant"
        )));
    }

    fn untraced(site: &str) -> UntracedTest {
        UntracedTest {
            site: site.into(),
            file: "RevokeTests.cs".into(),
        }
    }

    #[test]
    // covers: azimuth-rtm flag-untraced-test traced-test-without-scenario-untraced unit example
    fn an_untraced_test_under_a_traced_root_is_flagged() {
        let matrix = build(&[], &[], &[], &[], &[untraced("RevokeTests.SeedsFixtures")]);
        assert_eq!(matrix.holes.len(), 1);
        assert!(matches!(
            &matrix.holes[0].kind,
            HoleKind::UntracedTest { site } if site == "RevokeTests.SeedsFixtures"
        ));
    }

    // The opt-out (`[Untraced]`) and the area scope (a test outside every traced root) are applied
    // at emission — both simply withhold the entry — so at the core they reduce to "no entry, no
    // hole".
    #[test]
    fn a_manifest_with_no_untraced_entries_adds_no_holes() {
        let matrix = build(
            &[scenario("runs", unit_example())],
            &[],
            &[tag("runs", unit_example())],
            &[realization("runs")],
            &[],
        );
        assert!(matrix.is_whole());
    }

    #[test]
    // covers: azimuth-rtm scope-to-requested-specs references-closure-pulls-referenced unit example
    fn scope_closure_pulls_in_referenced_capabilities_transitively() {
        let invariants = [
            Invariant {
                spec_id: "public-certificates".into(),
                ..invariant("revoked-hidden", "public-cert", &["seo"])
            },
            Invariant {
                spec_id: "seo".into(),
                ..invariant("seo-inv", "seo-surface", &["search-index"])
            },
        ];
        let scope = scope_closure(&["public-certificates".to_string()], &invariants);
        assert!(scope.contains("public-certificates"));
        assert!(scope.contains("seo"));
        assert!(scope.contains("search-index"));
        assert!(!scope.contains("unrelated"));
    }
}
