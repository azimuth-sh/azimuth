#![allow(dead_code)]
//! The invented payouts product of the conditions tests. It is a third module beside the
//! sample's two, with keys declared in `execution-context.toml` and no component in the project
//! file, so no started component exists and every Check with a condition is owed.

use std::collections::BTreeSet;

use azimuth_tests::*;

pub const PAYOUT_KEY_FILE: &str = "azimuth/execution-context.toml";
pub const PAYOUTS_SPEC: &str = "azimuth/model/payouts/specification.md";
pub const PAYOUTS_VERIFICATION: &str = "azimuth/model/payouts/verification.md";
pub const PAYOUTS_TESTS: &str = "suites/desk-checks/payouts.test";

pub const SENT_AT_ONCE: &str = "payout-is-sent-at-once";
pub const REACHES_THE_CARD: &str = "payout-reaches-the-card";
pub const WAITS_FOR_REVIEW: &str = "payout-waits-for-review";
pub const PAYOUT_CHECKS: [&str; 2] = [REACHES_THE_CARD, WAITS_FOR_REVIEW];

pub const REACHES_CLAIM: &str = "An approved payout reaches the card at once.";
pub const WAITS_CLAIM: &str = "A payout under review is held until the review ends.";

pub const REQUIREMENT_HEADING: &str = "### Requirement: payout-is-sent-at-once\n";
pub const REACHES_HEADING: &str = "#### Scenario: payout-reaches-the-card\n";
pub const WAITS_HEADING: &str = "#### Scenario: payout-waits-for-review\n";
pub const REACHES_PROSE: &str = "The recipient sees it at once.\n";
pub const REQUIREMENT_PROSE: &str = "A payout is sent to the card as soon as it is approved.\n";

const SPEC: &str = "# Payouts\n\nMoney leaves the product only when a payout is approved.\n\n### Requirement: payout-is-sent-at-once\nA payout is sent to the card as soon as it is approved.\n\n#### Scenario: payout-reaches-the-card\nAn approved payout is sent to the card of its recipient.\nThe recipient sees it at once.\n\n#### Scenario: payout-waits-for-review\nAn approved payout of a recipient under review is held.\nThe recipient sees it after the review.\n";
const VERIFICATION: &str = "### Requirement: payout-is-sent-at-once\n#### Scenario: payout-reaches-the-card\n- Check: payout-reaches-the-card\n  An approved payout reaches the card at once.\n\n#### Scenario: payout-waits-for-review\n- Check: payout-waits-for-review\n  A payout under review is held until the review ends.\n";
const TESTS: &str = "test: sends-at-once\n# Check: payout-reaches-the-card\noutcome: passed\n\ntest: holds-for-review\n# Check: payout-waits-for-review\noutcome: passed\n";

/// The one-of values of `payouts.label`: spellings that a JSON string must escape, or that
/// RFC 8785 writes differently from how an author may write them.
pub const LABELS: [&str; 10] = [
    "a = b",
    "x=y",
    "  two  spaces",
    "a \"quoted\" b",
    "back\\slash",
    "tab\there",
    "line\nbreak",
    "\u{1f}",
    "a/b",
    "\u{e9}",
];

pub fn json_string(text: &str) -> String {
    canonical_json(&json!(text))
}

pub fn label_values() -> String {
    LABELS.iter().map(|label| json_string(label)).collect::<Vec<_>>().join(", ")
}

/// The keys of the payouts product. `payouts.region` lists `é` and `É` as two values, because no
/// case is folded.
pub fn payout_keys() -> String {
    format!(
        "[payouts.instant]\ntype = \"boolean\"\ncomponents = [\"api\", \"website\"]\n\n[payouts.region]\ntype = \"one of\"\nvalues = [\"no\", \"se\", \"\u{e9}\", \"\u{c9}\"]\ncomponents = [\"api\"]\n\n[payouts.limit]\ntype = \"number\"\ncomponents = [\"api\"]\n\n[payouts.label]\ntype = \"one of\"\nvalues = [{}]\ncomponents = [\"api\"]\n",
        label_values()
    )
}

/// The sample with the payouts module and its keys, committed. Its two payout Checks pass.
pub fn payouts() -> Repo {
    let repo = Repo::sample();
    repo.write(PAYOUTS_SPEC, SPEC);
    repo.write(PAYOUTS_VERIFICATION, VERIFICATION);
    repo.write(PAYOUTS_TESTS, TESTS);
    repo.write(PAYOUT_KEY_FILE, &payout_keys());
    repo.commit_all("payouts");
    repo
}

/// The payouts sample after one run, so every Check has the result it had before any condition.
pub fn payouts_with_results() -> Repo {
    let repo = payouts();
    repo.run_json(&[], 0);
    repo
}

pub fn all_checks() -> Vec<&'static str> {
    ALL_CHECKS.iter().copied().chain(PAYOUT_CHECKS).collect()
}

/// Writes lines right after a heading line of a file.
pub fn under(repo: &Repo, file: &str, heading: &str, lines: &[&str]) {
    let block: String = lines.iter().map(|line| format!("{line}\n")).collect();
    repo.replace(file, heading, &format!("{heading}{block}"));
}

pub fn on_requirement(repo: &Repo, lines: &[&str]) {
    under(repo, PAYOUTS_SPEC, REQUIREMENT_HEADING, lines);
}

pub fn on_reaches(repo: &Repo, lines: &[&str]) {
    under(repo, PAYOUTS_SPEC, REACHES_HEADING, lines);
}

pub fn on_waits(repo: &Repo, lines: &[&str]) {
    under(repo, PAYOUTS_SPEC, WAITS_HEADING, lines);
}

pub fn lint_is_clean(repo: &Repo) {
    let findings = repo.lint_findings(0);
    assert!(findings.is_empty(), "{findings:#?}");
}

/// Lint ends with a finding that names the specification file. Which line and which kind it
/// names is not fixed by the design.
pub fn lint_names_the_specification(repo: &Repo) -> Vec<serde_json::Value> {
    let findings = repo.lint_findings(1);
    assert!(
        !findings_at(&findings, PAYOUTS_SPEC, None).is_empty(),
        "no finding names {PAYOUTS_SPEC}: {findings:#?}"
    );
    findings
}

/// The reading commands answer although a condition has a finding.
pub fn status_answers(repo: &Repo) -> serde_json::Value {
    repo.status_json()
}

/// The claim text followed by the condition lines the design gives, as a claim part is computed
/// from them. Values are given in their normalized spelling; a one-of value in its JSON string
/// spelling. The lines are sorted by key here, so a caller passes them in any order.
pub fn claim_with(claim: &str, condition: &[(&str, &str)]) -> String {
    let mut lines: Vec<_> = condition.to_vec();
    lines.sort_by(|a, b| a.0.cmp(b.0));
    let mut text = claim.to_string();
    for (key, value) in lines {
        text.push_str(&format!("\nContext: {key}={value}"));
    }
    text
}

/// The claim part under the condition, exactly as the design computes it.
pub fn claim_digest(claim: &str, condition: &[(&str, &str)]) -> String {
    text_1_digest(&claim_with(claim, condition))
}

/// Rewrites the stored result of `check` as if the claim part of its fingerprint had been `claim`.
/// The fingerprint is recomputed from the four parts, so the record stays well formed and only
/// the claim differs.
pub fn store_with_claim(repo: &Repo, check: &str, claim: &str) {
    repo.rewrite_records(|record| {
        if record["format"] != "result-1" || record["subjects"][0]["name"] != check {
            return;
        }
        record["subjects"][0]["digest"] = json!(claim);
        record["body"]["parts"]["claim"] = json!(claim);
        let part = |name: &str| record["body"]["parts"][name].as_str().unwrap().to_string();
        let lines = format!(
            "claim {}\nfiles {}\nenvironment {}\nmethod {}\n",
            part("claim"),
            part("files"),
            part("environment"),
            part("method")
        );
        record["body"]["fingerprint"] = json!(format!("parts-1:{}", sha256_hex(lines.as_bytes())));
    });
}

/// The parts that `azimuth check show` names as changed against a stored result whose claim part
/// is the digest of `claim` under `condition`. The claim is among them exactly when the CLI
/// computes another claim part for the Check as the model stands now.
pub fn parts_changed_against(repo: &Repo, check: &str, claim: &str, condition: &[(&str, &str)]) -> BTreeSet<String> {
    store_with_claim(repo, check, &claim_digest(claim, condition));
    parts_changed_set(&repo.check_show(check))
}

/// The claim part of the Check equals the digest of its claim under exactly this condition.
pub fn claim_is(repo: &Repo, check: &str, claim: &str, condition: &[(&str, &str)]) {
    let parts = parts_changed_against(repo, check, claim, condition);
    assert!(
        !parts.contains("claim"),
        "{check}: the claim part is not the digest of {:?}: {parts:?}",
        claim_with(claim, condition)
    );
}

/// The claim part of the Check is not the digest of its claim under this condition.
pub fn claim_is_not(repo: &Repo, check: &str, claim: &str, condition: &[(&str, &str)]) {
    let parts = parts_changed_against(repo, check, claim, condition);
    assert!(
        parts.contains("claim"),
        "{check}: the claim part equals the digest of {:?}, which it must not: {parts:?}",
        claim_with(claim, condition)
    );
}

pub fn result_claim(repo: &Repo, check: &str) -> String {
    repo.result_of(check)["body"]["parts"]["claim"].as_str().unwrap_or_default().to_string()
}

/// The Check is owed with the reason "context not applied" and has no result.
pub fn owed_not_applied(repo: &Repo, document: &serde_json::Value, check: &str) {
    assert_eq!(state_of(document, check), "owed", "{check}: {document:#}");
    assert_eq!(reason_of(document, check), "context not applied", "{check}: {document:#}");
    assert!(repo.results_of(check).is_empty(), "{check} has a result: {:#?}", repo.results_of(check));
}

/// The claim part of the Check is the digest of exactly this text, with no rule of ours applied
/// to it: for the controls that test what must not be normalized or reordered.
pub fn claim_is_raw(repo: &Repo, check: &str, text: &str) {
    store_with_claim(repo, check, &text_1_digest(text));
    let parts = parts_changed_set(&repo.check_show(check));
    assert!(!parts.contains("claim"), "{check}: the claim part is not the digest of {text:?}: {parts:?}");
}

pub fn claim_is_not_raw(repo: &Repo, check: &str, text: &str) {
    store_with_claim(repo, check, &text_1_digest(text));
    let parts = parts_changed_set(&repo.check_show(check));
    assert!(parts.contains("claim"), "{check}: the claim part equals the digest of {text:?}, which it must not: {parts:?}");
}

/// Conditions that read well and that no started component can apply: none is started in this
/// product, so any condition qualifies. Each pair is the lines for the Requirement and the lines
/// for the Scenario `payout-reaches-the-card`.
pub const FAULTY_CONDITIONS: [(&[&str], &[&str]); 5] = [
    (&["Context: payouts.speed=true"], &[]),
    (&["Context: payouts.instant=\"true\""], &[]),
    (&["Context: payouts.region=\"xx\""], &[]),
    (&["Context: payouts.instant=true", "Context: payouts.instant=false"], &[]),
    (&["Context: payouts.instant=true"], &["Context: payouts.instant=false"]),
];

pub fn faulty_payouts(requirement: &[&str], scenario: &[&str]) -> Repo {
    let repo = payouts();
    on_requirement(&repo, requirement);
    if !scenario.is_empty() {
        on_reaches(&repo, scenario);
    }
    repo
}

/// Every Check of the sample is promised under the same declared condition, so no Check of the
/// repository is free of it.
pub fn every_check_under_a_condition() -> Repo {
    let repo = payouts();
    on_requirement(&repo, &["Context: payouts.instant=true"]);
    repo.replace(LENDING_SPEC, "### Requirement: loan-limit-holds\n", "### Requirement: loan-limit-holds\nContext: payouts.instant=true\n");
    repo.replace(RETURNS_SPEC, "### Requirement: late-return-owes-a-fine\n", "### Requirement: late-return-owes-a-fine\nContext: payouts.instant=true\n");
    repo
}
