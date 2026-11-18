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

pub const RUNNERS: [Runner; 2] = [
    Runner {
        stack: ".NET with xUnit",
        project_file_endings: &[".csproj", ".fsproj", ".vbproj"],
        reference: &["include=\"xunit\"", "include=\"xunit.v3\""],
        marker: "[Trait(\"Check\", \"<id>\")]",
        adapter: "dotnet azimuth-xunit",
        install_command: "dotnet tool install --local --create-manifest-if-needed Azimuth.Xunit",
    },
    Runner {
        stack: "Playwright",
        project_file_endings: &["package.json"],
        reference: &["@playwright/test"],
        marker: "@Check:<id>",
        adapter: "npx --no @azimuth-sh/playwright",
        install_command: "npm install --save-dev --save-exact @azimuth-sh/playwright",
    },
];

impl Runner {
    /// The CLI and its adapters are released together under one version, and the version must be
    /// named, because without it `dotnet` skips a pre-release.
    pub fn install_line(&self) -> String {
        if self.project_file_endings == ["package.json"] {
            format!("{}@{}", self.install_command, crate::VERSION)
        } else {
            format!("{} --version {}", self.install_command, crate::VERSION)
        }
    }

    /// Whether a file of the tree shows that the stack exists.
    pub fn recognizes(&self, path: &str, text: &str) -> bool {
        if !self.project_file_endings.iter().any(|ending| path.ends_with(ending)) {
            return false;
        }
        if self.project_file_endings == ["package.json"] {
            return path.rsplit('/').next() == Some("package.json")
                && self.reference.iter().any(|package| Self::lists_package(text, package));
        }
        let text = text.to_lowercase();
        self.reference.iter().any(|reference| text.contains(reference))
    }

    pub fn lists_adapter(&self, path: &str, text: &str) -> bool {
        path.rsplit('/').next() == Some("package.json")
            && self.adapter.strip_prefix("npx --no ").is_some_and(|package| Self::lists_package(text, package))
    }

    fn lists_package(text: &str, package: &str) -> bool {
        let Ok(manifest) = serde_json::from_str::<serde_json::Value>(text) else {
            return false;
        };
        ["dependencies", "devDependencies"].iter().any(|section| manifest[section].get(package).is_some())
    }
}

/// The install line of a known adapter command.
pub fn install_line(adapter: &str) -> Option<String> {
    let words: Vec<&str> = adapter.split(' ').filter(|word| !word.is_empty()).collect();
    RUNNERS.iter().find(|runner| words.starts_with(&runner.adapter.split(' ').collect::<Vec<_>>())).map(Runner::install_line)
}
