// Tipos y llamadas al backend de Rust (src-tauri/src/comandos.rs).
import { invoke as invocarTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invocarSimulado } from './simulacion'

// Fuera de Tauri (solo en desarrollo) se usan datos de ejemplo para poder ver las pantallas.
const invoke = <T>(comando: string, args?: Record<string, unknown>): Promise<T> =>
  '__TAURI_INTERNALS__' in window || !(import.meta.env.DEV || import.meta.env.VITE_SIMULACION === '1') ? invocarTauri<T>(comando, args) : invocarSimulado<T>(comando)

export type Origen = 'estado' | 'steam' | 'registro' | 'manual'

export interface JuegoDetectado {
  nombre: string
  appId: string | null
  ruta: string
  version: string | null
  origen: Origen
}

export interface ValidacionJuego {
  ok: boolean
  ruta: string | null
  version: string | null
  versionCompatible: boolean | null
  errores: string[]
  avisos: string[]
}

export interface ArchivoAjeno {
  ruta: string
  tamano: number
}

export type EstadoCarpeta =
  | { tipo: 'vacia' }
  | { tipo: 'parche'; versiones: string[]; completo: boolean }
  | { tipo: 'ajenos'; archivos: ArchivoAjeno[] }

export interface ValidacionDatos {
  ok: boolean
  rutaNormalizada: string
  existe: boolean
  errores: string[]
  avisos: string[]
  estado: EstadoCarpeta | null
  espacioLibre: number | null
  espacioNecesario: number
}

export interface Estado {
  patchVersion: string
  dataPath: string
  pathMode: 'auto' | 'manual'
  gamePath: string | null
  gameVersion: string | null
  backup: boolean
  backupPath: string | null
  autoUpdate: boolean
  updateIntervalHours: number
  shortcut: boolean
  installedAt: string
  lastCheck: string | null
  lastCheckResult: string | null
  unsupportedGameVersion: string | null
  targets: string[]
  files: Record<string, string>
  createdDirs: string[]
}

export interface Deteccion {
  juego: JuegoDetectado | null
  validacionJuego: ValidacionJuego | null
  rutaDatos: string | null
  validacionDatos: ValidacionDatos | null
  estado: Estado | null
  archivosDanados: string[]
  actualizacionProgramada: boolean
  juegoAbierto: boolean
  localAppData: string | null
}

export interface InfoPaquete {
  version: string
  versionInstalador: string
  archivos: number
  copias: number
  tamanoTotal: number
  tamanoInstalado: number
  targets: string[]
  gameVersions: string[]
  urlReleases: string
  urlReportar: string
  intervalos: number[]
  intervaloPorDefecto: number
}

export interface InfoActualizacion {
  versionInstalada: string | null
  versionDisponible: string
  hayParcheNuevo: boolean
  versionJuego: string | null
  compatibleConJuego: boolean | null
  notas: string | null
  tamanoDescarga: number
  versionInstalador: string
  instaladorDisponible: string | null
  hayInstaladorNuevo: boolean
}

export interface Opciones {
  rutaDatos: string
  modoRuta: 'auto' | 'manual'
  rutaJuego: string | null
  copiaSeguridad: boolean
  autoActualizar: boolean
  intervaloActualizacionHoras: number
  accesoDirecto: boolean
  dryRun?: boolean
  aceptarVersionNoProbada?: boolean
}

export interface Progreso {
  paso: string
  porcentaje: number
  mensaje: string
}

export interface ResultadoInstalacion {
  dryRun: boolean
  version: string
  rutaDatos: string
  archivosEscritos: number
  archivosSinCambios: number
  carpetasCreadas: string[]
  copiaSeguridad: string | null
  archivosRespaldados: string[]
  acciones: string[]
  avisos: string[]
}

export interface ResultadoDesinstalacion {
  dryRun: boolean
  borrados: string[]
  conservados: string[]
  restaurados: string[]
  carpetasQuitadas: string[]
  acciones: string[]
  avisos: string[]
}

export interface ArgumentosInicio {
  accion: 'asistente' | 'instalar' | 'desinstalar' | 'reparar' | 'actualizar'
  dryRun: boolean
}

export const api = {
  argumentosInicio: () => invoke<ArgumentosInicio>('argumentos_inicio'),
  infoPaquete: () => invoke<InfoPaquete>('info_paquete'),
  detectar: () => invoke<Deteccion>('detectar'),
  validarJuego: (ruta: string) => invoke<ValidacionJuego>('validar_juego', { ruta }),
  validarDatos: (ruta: string) => invoke<ValidacionDatos>('validar_datos', { ruta }),
  juegoAbierto: () => invoke<boolean>('juego_abierto'),
  abrirJuego: (appId: string | null, ruta: string | null) => invoke<void>('abrir_juego', { appId, ruta }),
  instalar: (opciones: Opciones) => invoke<ResultadoInstalacion>('instalar', { opciones }),
  reparar: (dryRun = false) => invoke<ResultadoInstalacion>('reparar', { dryRun }),
  desinstalar: (borrarBackup: boolean, forzar = false, dryRun = false) =>
    invoke<ResultadoDesinstalacion>('desinstalar', { borrarBackup, forzar, dryRun }),
  buscarActualizacion: () => invoke<InfoActualizacion>('buscar_actualizacion'),
  aplicarActualizacion: () => invoke<ResultadoInstalacion>('aplicar_actualizacion'),
  actualizarInstalador: () => invoke<string>('actualizar_instalador'),
  cambiarOpciones: (autoUpdate: boolean, intervaloHoras: number) =>
    invoke<Estado>('cambiar_opciones', { autoUpdate, intervaloHoras }),
  abrirUrl: (url: string) => invoke<void>('abrir_url', { url }),
  alProgresar: (f: (p: Progreso) => void): Promise<UnlistenFn> =>
    enTauri()
      ? listen<Progreso>('progreso', (e) => f(e.payload))
      : (f({ paso: 'copiando', porcentaje: 62, mensaje: 'Instalando en epk (60/111)…' }), Promise.resolve(() => {})),
}

export const enTauri = () => '__TAURI_INTERNALS__' in window

export function formatoMB(bytes: number): string {
  const mb = bytes / (1024 * 1024)
  return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${mb.toFixed(1)} MB`
}

export function nombreIntervalo(horas: number): string {
  switch (horas) {
    case 6:
      return 'Cada 6 horas'
    case 12:
      return 'Cada 12 horas'
    case 24:
      return 'Una vez al día (recomendado)'
    case 168:
      return 'Una vez a la semana'
    default:
      return `Cada ${horas} horas`
  }
}

export function formatoFecha(iso: string | null): string {
  if (!iso) return 'nunca'
  const f = new Date(iso)
  if (Number.isNaN(f.getTime())) return iso
  return f.toLocaleString('es', { dateStyle: 'medium', timeStyle: 'short' })
}

export const mensajeError = (e: unknown) => (typeof e === 'string' ? e : e instanceof Error ? e.message : String(e))
