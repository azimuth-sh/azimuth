//! The method validating itself: parse `rtm`'s own spec, scan its own source for `covers` and
//! `realizes` tags, and assert the matrix has no holes. Adding a requirement without tagging it,
//! or covering a completeness scenario with only an example, breaks this test.

use azimuth_rtm::{build, scan, spec};

#[test]
fn azimuth_rtm_regenerates_a_matrix_with_no_holes() {
    let manifest = env!("CARGO_MANIFEST_DIR");

    let spec_text = std::fs::read_to_string(format!("{manifest}/../openspec/specs/rtm/spec.md"))
        .expect("rtm spec is readable");
    let parsed = spec::parse_spec(&spec_text);
    assert!(
        !parsed.scenarios.is_empty(),
        "the spec should declare scenarios"
    );

    let (tags, realizations) = scan::scan_dir(&format!("{manifest}/src"));

    let matrix = build(&parsed.scenarios, &parsed.invariants, &tags, &realizations);

    assert!(
        matrix.is_whole(),
        "azimuth-rtm must have no holes, found: {:?}",
        matrix.holes
    );
}
