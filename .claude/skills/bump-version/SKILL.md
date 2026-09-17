---
name: bump-version
description: Use when preparing a Clip2Pod release and the version number needs to move — bumps package.json, src-tauri/tauri.conf.json, and src-tauri/Cargo.toml together so they never drift out of sync.
disable-model-invocation: true
---

# bump-version

Clip2Pod's version string lives in three files that must always match:
`package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`. Bumping
only some of them produces a release build tagged with the wrong version.

## Usage

```
node .claude/skills/bump-version/bump.cjs <new-version>
```

This reads the current version from all three files, refuses to proceed if
they're already inconsistent, writes the new version to each, and prints a
suggested commit message matching this repo's convention:

```
chore(release): bump version to <new-version>
```

Review the diff (`git diff`), then commit with that message and tag as usual.
