# Installation account

`azimuth/installation.json` is the tracked ownership account for one consumer repository. Version 1 is strict JSON with no inferred resources or components.

```json
{
  "format": "azimuth-installation",
  "schemaVersion": 1,
  "releaseVersion": "0.1.0-alpha.5",
  "migrationLine": "alpha-claim-case",
  "agents": ["claude", "codex"],
  "components": [
    {
      "id": "typescript-annotations",
      "manifest": "frontend/package.json",
      "version": "0.1.0-alpha.5"
    }
  ],
  "resources": [
    {
      "id": "skill:codex:azimuth-propose",
      "path": ".agents/skills/azimuth-propose/SKILL.md",
      "sha256": "<64 lowercase hexadecimal characters>"
    }
  ],
  "aliases": [
    {
      "integration": "claude",
      "path": ".claude/skills",
      "target": "../../.agents/skills"
    }
  ]
}
```

`releaseVersion` identifies the CLI-managed cohort, while `migrationLine` identifies supported semantic migration composition. Agents are sorted unique members of `claude | codex`. Component ids are sorted unique members of `typescript-annotations | typescript-emitter | dotnet-annotations | dotnet-emitter`; each manifest is a non-escaping repository-relative native dependency manifest and each version is the exact registered cohort version.

Resources have unique ids and paths. Paths are non-escaping and repository-relative. The raw lowercase SHA-256 covers the complete installed bytes. An alias is accepted only through explicit initialization, remains team-owned, uses a relative target and resolves inside the repository to the supported `.agents/skills` location. Azimuth never creates an alias.

The CLI writes this file. Agents and users change it through `azimuth init`, `azimuth agent`, `azimuth component` and `azimuth update`, not by manual editing.
