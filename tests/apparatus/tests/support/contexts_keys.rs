use azimuth_tests::*;

pub const KEY_FILE: &str = "azimuth/execution-context.toml";
pub const INSTANT: &str = "[payouts.instant]\ntype = \"boolean\"\ncomponents = [\"api\", \"website\"]\n";

pub fn declare_components(repo: &Repo) {
    repo.append(
        PROJECT_FILE,
        "\n[[component]]\nname = \"api\"\nareas = [\"all\"]\nreport = \"/internal/azimuth\"\n\n[[component]]\nname = \"website\"\nareas = [\"all\"]\nreport = \"/internal/azimuth\"\n",
    );
}
