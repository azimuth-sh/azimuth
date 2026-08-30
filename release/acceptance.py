APPROVED_CONTRACT = {
    "version": "0.1.0-alpha.6",
    "tag": "v0.1.0-alpha.6",
    "migrationLine": "alpha-claim-case",
    "license": "Apache-2.0",
    "repository": "https://github.com/azimuth-sh/azimuth",
    "homepage": "https://azimuth.sh",
    "identities": sorted(
        [
            "azimuth",
            "Azimuth.Annotations",
            "Azimuth.Emit",
            "@azimuth-sh/annotations",
            "@azimuth-sh/emit",
            "ghcr.io/azimuth-sh/azimuth-assurance-api",
            "ghcr.io/azimuth-sh/azimuth-assurance-web",
        ]
    ),
    "nativeTargets": sorted(
        [
            "x86_64-unknown-linux-gnu",
            "aarch64-apple-darwin",
            "x86_64-pc-windows-msvc",
        ]
    ),
    "imagePlatforms": {
        "ghcr.io/azimuth-sh/azimuth-assurance-api": ["linux/amd64", "linux/arm64"],
        "ghcr.io/azimuth-sh/azimuth-assurance-web": ["linux/amd64", "linux/arm64"],
    },
    "supportedSurfaces": sorted(
        [
            "rust-cli-core",
            "dotnet-integration",
            "typescript-integration",
            "assurance-reference",
            "repository-contracts",
            "consumer-installation",
        ]
    ),
    "experimentalSource": sorted(
        [
            "experiments",
            "packages/cpp",
            "packages/go",
            "packages/jvm",
            "packages/python",
            "packages/rust",
            "tools/extractors/cpp",
            "tools/extractors/go",
            "tools/extractors/jvm",
            "tools/extractors/python",
            "tools/extractors/rust",
        ]
    ),
}
