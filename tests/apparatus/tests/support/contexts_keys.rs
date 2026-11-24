// A key's components are declared in the project file, so a fixture that expects a clean
// lint declares them beside the key (`declare_payout_components`).
pub const KEY_FILE: &str = "azimuth/execution-context.toml";
pub const INSTANT: &str = "[payouts.instant]\ntype = \"boolean\"\ncomponents = [\"api\", \"website\"]\n";
