APPROVED_CONTRACT = {
    "version": "0.1.0-alpha.1",
    "tag": "v0.1.0-alpha.1",
    "license": "Apache-2.0",
    "identities": sorted(
        [
            "azimuth",
            "Azimuth.Annotations",
            "Azimuth.Emit",
            "@azimuth/annotations",
            "@azimuth/emit",
            "ghcr.io/drim-dev/azimuth-assurance-api",
            "ghcr.io/drim-dev/azimuth-assurance-web",
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
        "ghcr.io/drim-dev/azimuth-assurance-api": ["linux/amd64", "linux/arm64"],
        "ghcr.io/drim-dev/azimuth-assurance-web": ["linux/amd64", "linux/arm64"],
    },
    "supportedSurfaces": sorted(
        [
            "rust-cli-core",
            "dotnet-integration",
            "typescript-integration",
            "assurance-reference",
            "repository-contracts",
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
