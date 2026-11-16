//! The small table of the runners the CLI knows. It is how `azimuth init` sees a stack whose
//! adapter is absent, and the only knowledge of a stack the CLI carries.

pub struct Runner {
    /// The stack, as a person names it.
    pub stack: &'static str,
    /// File names that may be a project file of the runner.
    pub project_file_endings: &'static [&'static str],
    /// Text whose presence in such a project file shows that it references the runner.
    pub reference: &'static [&'static str],
    pub marker: &'static str,
    pub adapter: &'static str,
    /// The install command without its version, which is always this CLI's own.
    install_command: &'static str,
}

pub const RUNNERS: [Runner; 1] = [Runner {
    stack: ".NET with xUnit",
    project_file_endings: &[".csproj", ".fsproj", ".vbproj"],
    reference: &["include=\"xunit\"", "include=\"xunit.v3\""],
    marker: "[Trait(\"Check\", \"<id>\")]",
    adapter: "dotnet azimuth-xunit",
    install_command: "dotnet tool install --local --create-manifest-if-needed Azimuth.Xunit",
}];

impl Runner {
    /// The CLI and its adapters are released together under one version, and the version must be
    /// named, because without it `dotnet` skips a pre-release.
    pub fn install_line(&self) -> String {
        format!("{} --version {}", self.install_command, crate::VERSION)
    }

    /// Whether a file of the tree shows that the stack exists.
    pub fn recognizes(&self, path: &str, text: &str) -> bool {
        if !self.project_file_endings.iter().any(|ending| path.ends_with(ending)) {
            return false;
        }
        let text = text.to_lowercase();
        self.reference.iter().any(|reference| text.contains(reference))
    }
}

/// The install line of a known adapter command.
pub fn install_line(adapter: &str) -> Option<String> {
    let words: Vec<&str> = adapter.split(' ').filter(|word| !word.is_empty()).collect();
    RUNNERS.iter().find(|runner| runner.adapter.split(' ').eq(words.iter().copied())).map(Runner::install_line)
}
