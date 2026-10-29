/**
 * Sync the package version across:
 *  - root package.json (source of truth)
 *  - root Cargo.toml [workspace.package] version
 *  - all optionalDependencies entries in root package.json
 *  - crates/dsp-node/package.json (if it exists)
 *
 * Run automatically by `npm version <bump>` via the "version" lifecycle script.
 *
 * Usage:
 *   node scripts/sync-versions.js
 *   npm version patch    # automatically calls this
 */

const { readFileSync, writeFileSync, existsSync } = require('node:fs')
const { join } = require('node:path')

const ROOT = join(__dirname, '..')

// ─── Source of truth: root package.json ─────────────────────────────────────

const ROOT_PKG_PATH = join(ROOT, 'package.json')
const ROOT_PKG = JSON.parse(readFileSync(ROOT_PKG_PATH, 'utf8'))
const VERSION = ROOT_PKG.version

console.log(`syncing version → ${VERSION}`)

// ─── 1. Sync optionalDependencies in root package.json ──────────────────────

let optionalDepsUpdated = 0
if (ROOT_PKG.optionalDependencies) {
  for (const dep of Object.keys(ROOT_PKG.optionalDependencies)) {
    if (dep.startsWith('@kryxjs/dsp-')) {
      ROOT_PKG.optionalDependencies[dep] = VERSION
      optionalDepsUpdated += 1
    }
  }
  writeFileSync(ROOT_PKG_PATH, `${JSON.stringify(ROOT_PKG, null, 2)}\n`)
  console.log(`  ✓ root package.json — synced ${optionalDepsUpdated} optionalDependencies`)
}

// ─── 2. Sync Cargo.toml [workspace.package] version ─────────────────────────

const CARGO_TOML_PATH = join(ROOT, 'Cargo.toml')
if (existsSync(CARGO_TOML_PATH)) {
  let cargo = readFileSync(CARGO_TOML_PATH, 'utf8')
  const before = cargo
  // Match the FIRST `version = "X.Y.Z"` line (which is the workspace one)
  cargo = cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${VERSION}"`)
  if (cargo !== before) {
    writeFileSync(CARGO_TOML_PATH, cargo)
    console.log(`  ✓ Cargo.toml — workspace.package.version → ${VERSION}`)
  } else {
    console.log('  ⚠ Cargo.toml — no version field changed (already in sync?)')
  }
} else {
  console.log(`  ⚠ Cargo.toml not found at ${CARGO_TOML_PATH}, skipping`)
}

// ─── 3. Sync crates/dsp-node/package.json IF it exists ──────────────────────

const DSPNODE_PKG_PATH = join(ROOT, 'crates', 'dsp-node', 'package.json')
if (existsSync(DSPNODE_PKG_PATH)) {
  const DSPNODE_PKG = JSON.parse(readFileSync(DSPNODE_PKG_PATH, 'utf8'))
  DSPNODE_PKG.version = VERSION
  writeFileSync(DSPNODE_PKG_PATH, `${JSON.stringify(DSPNODE_PKG, null, 2)}\n`)
  console.log(`  ✓ crates/dsp-node/package.json — version → ${VERSION}`)
}

console.log('done.')
