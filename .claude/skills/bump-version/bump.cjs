#!/usr/bin/env node
// Bumps the version string across the three files that must move together in
// this repo: package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml.
const fs = require('fs');
const path = require('path');

const newVersion = process.argv[2];
if (!newVersion || !/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error('Usage: node bump.cjs <new-version>  (e.g. 0.9.0)');
  process.exit(1);
}

const root = path.resolve(__dirname, '..', '..', '..');
const targets = [
  { file: 'package.json', pattern: /"version":\s*"(\d+\.\d+\.\d+)"/ },
  { file: 'src-tauri/tauri.conf.json', pattern: /"version":\s*"(\d+\.\d+\.\d+)"/ },
  { file: 'src-tauri/Cargo.toml', pattern: /^version = "(\d+\.\d+\.\d+)"/m },
];

const found = targets.map(({ file, pattern }) => {
  const p = path.join(root, file);
  const content = fs.readFileSync(p, 'utf8');
  const match = content.match(pattern);
  if (!match) throw new Error(`Couldn't find a version field in ${file}`);
  return { path: p, rel: file, content, old: match[1], pattern };
});

const oldVersions = new Set(found.map((f) => f.old));
if (oldVersions.size > 1) {
  console.error(`Versions are already inconsistent: ${[...oldVersions].join(', ')}. Fix that by hand first.`);
  process.exit(1);
}

for (const { path: p, rel, content, old, pattern } of found) {
  const updated = content.replace(pattern, (m) => m.replace(old, newVersion));
  fs.writeFileSync(p, updated);
  console.log(`${rel}: ${old} -> ${newVersion}`);
}

console.log(`\nCommit with: chore(release): bump version to ${newVersion}`);
