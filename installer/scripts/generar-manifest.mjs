// Genera el manifest.json que se sube a cada release (§8 del plan).
//
//   node scripts/generar-manifest.mjs --zip <GALAXYZ-neo-Espanol-Latino.zip> [--instalador <GalaxyzNeoES.exe>]
//        [--anterior <manifest.json de la release anterior>] [--notas "texto"] [--salida manifest.json]
//
// Toma la versión, las versiones del juego y las carpetas de src-tauri/payload/manifest.base.json,
// los .epk de la carpeta epk/ del repositorio y guarda los hashes de la versión anterior en
// knownPatchHashes para que el instalador los reconozca como del parche.
import { createHash } from 'node:crypto'
import { readFileSync, readdirSync, statSync, writeFileSync, existsSync } from 'node:fs'
import { basename, dirname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const aqui = dirname(fileURLToPath(import.meta.url))
const raizInstalador = resolve(aqui, '..')
const raizRepo = resolve(raizInstalador, '..')

const args = process.argv.slice(2)
const opcion = (nombre) => {
  const i = args.indexOf(nombre)
  return i >= 0 ? args[i + 1] : undefined
}

const sha256 = (buf) => createHash('sha256').update(buf).digest('hex')

const base = JSON.parse(readFileSync(join(raizInstalador, 'src-tauri', 'payload', 'manifest.base.json'), 'utf8'))
const carpetaEpk = resolve(opcion('--epk') ?? join(raizRepo, 'epk'))
const files = readdirSync(carpetaEpk)
  .filter((n) => n.toLowerCase().endsWith('.epk'))
  .sort()
  .map((name) => {
    const datos = readFileSync(join(carpetaEpk, name))
    return { name, size: datos.length, sha256: sha256(datos) }
  })

// extra/: archivos con ruta propia dentro de data\ (p. ej. imágenes traducidas). Igual que build.rs.
const carpetaExtra = resolve(opcion('--extra') ?? join(raizRepo, 'extra'))
const listar = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? listar(join(dir, e.name)) : [join(dir, e.name)],
  )
if (existsSync(carpetaExtra)) {
  for (const ruta of listar(carpetaExtra).sort()) {
    const name = basename(ruta)
    if (name === '.gitkeep' || name === 'LEEME.md') continue
    const datos = readFileSync(ruta)
    const path = relative(carpetaExtra, ruta).replaceAll('/', '\\')
    files.push({ name, path, size: datos.length, sha256: sha256(datos) })
  }
}

const zipRuta = opcion('--zip')
if (!zipRuta || !existsSync(zipRuta)) {
  console.error('Falta --zip con la ruta del ZIP del parche.')
  process.exit(1)
}
const zipDatos = readFileSync(zipRuta)

const manifest = {
  version: opcion('--version') ?? base.version,
  released: new Date().toISOString().slice(0, 10),
  gameVersions: base.gameVersions,
  targets: base.targets,
  files,
  knownPatchHashes: { ...(base.knownPatchHashes ?? {}) },
  zip: { url: basename(zipRuta), sha256: sha256(zipDatos), size: zipDatos.length },
}

const anterior = opcion('--anterior')
if (anterior) {
  const m = JSON.parse(readFileSync(anterior, 'utf8'))
  Object.assign(manifest.knownPatchHashes, m.knownPatchHashes ?? {})
  if (m.version !== manifest.version) {
    manifest.knownPatchHashes[m.version] = Object.fromEntries(m.files.map((f) => [f.path ?? f.name, f.sha256]))
  }
}

const instalador = opcion('--instalador')
if (instalador) {
  const datos = readFileSync(instalador)
  const cargo = readFileSync(join(raizInstalador, 'src-tauri', 'Cargo.toml'), 'utf8')
  const version = /^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1]
  manifest.installer = { version, url: basename(instalador), sha256: sha256(datos), size: statSync(instalador).size }
}

// Notas cortas para Mantenimiento: --notas "texto" o el primer párrafo de --notas-archivo (sin títulos).
let notas = opcion('--notas')
const archivoNotas = opcion('--notas-archivo')
if (!notas && archivoNotas && existsSync(archivoNotas)) {
  notas = readFileSync(archivoNotas, 'utf8')
    .split(/\r?\n\r?\n/)
    .map((p) => p.trim())
    .find((p) => p && !p.startsWith('#'))
    ?.replace(/\s+/g, ' ')
    .replace(/[*_`]/g, '')
    .slice(0, 300)
}
if (notas) manifest.notes = notas

const salida = opcion('--salida') ?? 'manifest.json'
writeFileSync(salida, JSON.stringify(manifest, null, 2) + '\n')
console.log(`manifest.json v${manifest.version}: ${files.length} archivos → ${salida}`)
