//! `code-map <source-root>...` — the derivable half of a code-map: scan `realizes` tags and print,
//! per code site, the scenarios it is on the path of. This is orientation reconstructed from the
//! code ("what is this site for"), the inverse of the rtm matrix's spec-first view. The judgment
//! half of a code-map — danger zones, intentional corners, what is deliberately absent — is not
//! derivable and stays hand-written.

use azimuth_rtm::scan;
use std::collections::BTreeMap;

fn main() {
    let roots: Vec<String> = std::env::args().skip(1).collect();
    if roots.is_empty() {
        eprintln!("usage: code-map <source-root>...");
        std::process::exit(64);
    }

    let mut by_site: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for root in &roots {
        let (_tags, realizations) = scan::scan_dir(root);
        for realization in realizations {
            by_site.entry(realization.site).or_default().push(format!(
                "{}/{}/{}",
                realization.key.spec_id, realization.key.req_id, realization.key.scenario_id
            ));
        }
    }

    println!("# Code map (derived)\n");
    if by_site.is_empty() {
        println!("_no realized sites found_");
        return;
    }

    for (site, mut scenarios) in by_site {
        scenarios.sort();
        println!("## {site}");
        for scenario in scenarios {
            println!("- realizes {scenario}");
        }
        println!();
    }
}
