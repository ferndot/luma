// Runs the Tauri CLI from apps/desktop/src-tauri, where tauri.conf.json lives.
// A plain Node script instead of `cd ... && ...` so it works the same on Windows, macOS and Linux.
// Usage: node scripts/tauri.mjs dev | build | <any tauri args>
import { spawn } from 'node:child_process'
import { createRequire } from 'node:module'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const appDir = path.join(root, 'apps', 'desktop', 'src-tauri')
const uiDir = path.join(root, 'apps', 'desktop', 'ui')

const require = createRequire(path.join(uiDir, 'package.json'))
let cli
try {
  cli = require.resolve('@tauri-apps/cli/tauri.js')
} catch {
  console.error('Tauri CLI not found. Install UI dependencies first: pnpm --dir apps/desktop/ui install')
  process.exit(1)
}

const child = spawn(process.execPath, [cli, ...process.argv.slice(2)], { cwd: appDir, stdio: 'inherit' })
child.on('exit', (code, signal) => process.exit(code ?? (signal ? 1 : 0)))
