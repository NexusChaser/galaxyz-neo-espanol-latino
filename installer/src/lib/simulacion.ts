// SOLO DESARROLLO: datos de ejemplo para ver las pantallas en un navegador sin Tauri.
// Uso: http://localhost:5173/?pantalla=deteccion  (bienvenida, deteccion, noEncontrado, opciones,
// resumen, instalando, listo, mantenimiento). Con &ajenos=1, &juegoViejo=1 o &sinAuto=1 cambia el caso.
import type { Deteccion, InfoActualizacion, InfoPaquete } from './api'

const p = new URLSearchParams(location.search)

const info: InfoPaquete = {
  version: '1.0.0',
  versionInstalador: '1.0.0',
  archivos: 37,
  copias: 111,
  tamanoTotal: 21_630_000,
  tamanoInstalado: 64_890_000,
  targets: ['locale\\us\\epk', 'epk', 'root\\epk'],
  gameVersions: ['20260210_102029'],
  urlReleases: 'https://github.com/NexusChaser/galaxyz-neo-espanol-latino/releases/latest',
  urlReportar: 'https://github.com/NexusChaser/galaxyz-neo-espanol-latino/issues/new/choose',
  intervalos: [6, 12, 24, 168],
  intervaloPorDefecto: 24,
}

const instalado = p.get('pantalla') === 'mantenimiento'
const datos = 'C:\\Users\\migue\\AppData\\Local\\fuzz\\galaxyz\\data'

const deteccion: Deteccion = {
  juego:
    p.get('pantalla') === 'noEncontrado'
      ? null
      : {
          nombre: 'GALAXYZ neo Demo',
          appId: '4306970',
          ruta: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\GALAXYZ neo Demo',
          version: p.get('juegoViejo') ? '20261001_000000' : '20260210_102029',
          origen: 'steam',
        },
  validacionJuego: {
    ok: true,
    ruta: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\GALAXYZ neo Demo',
    version: p.get('juegoViejo') ? '20261001_000000' : '20260210_102029',
    versionCompatible: !p.get('juegoViejo'),
    errores: [],
    avisos: p.get('juegoViejo')
      ? ['La versión del juego (20261001_000000) no se probó con este parche. Puede haber textos raros o cierres.']
      : [],
  },
  rutaDatos: datos,
  validacionDatos: {
    ok: true,
    rutaNormalizada: datos,
    existe: true,
    errores: [],
    avisos: [],
    estado: p.get('ajenos')
      ? { tipo: 'ajenos', archivos: [{ ruta: 'epk\\scene_data.epk', tamano: 17_000_000 }, { ruta: 'epk\\item_data.epk', tamano: 150_000 }] }
      : { tipo: 'vacia' },
    espacioLibre: 120_000_000_000,
    espacioNecesario: 75_000_000,
  },
  estado: instalado
    ? {
        patchVersion: '1.0.0',
        dataPath: datos,
        pathMode: 'auto',
        gamePath: null,
        gameVersion: '20260210_102029',
        backup: true,
        backupPath: null,
        autoUpdate: !p.get('sinAuto'),
        updateIntervalHours: 24,
        shortcut: true,
        installedAt: '2026-09-13T18:00:00-05:00',
        lastCheck: '2026-09-14T09:12:00-05:00',
        lastCheckResult: 'Hay una versión nueva: 1.1.0',
        targets: info.targets,
        files: {},
        createdDirs: [],
        unsupportedGameVersion: p.get('juegoViejo') ? '20261001_000000' : null,
      }
    : null,
  archivosDanados: p.get('danados') ? ['epk\\commontext.epk'] : [],
  actualizacionProgramada: true,
  juegoAbierto: false,
  localAppData: 'C:\\Users\\migue\\AppData\\Local',
}

const actualizacion: InfoActualizacion = {
  versionInstalada: '1.0.0',
  versionDisponible: '1.1.0',
  hayParcheNuevo: true,
  versionJuego: '20260210_102029',
  compatibleConJuego: true,
  notas: 'Correcciones en diálogos del capítulo 3 y en la tienda de canjes.',
  tamanoDescarga: 9_800_000,
  versionInstalador: '1.0.0',
  instaladorDisponible: '1.0.1',
  hayInstaladorNuevo: true,
}

const respuestas: Record<string, unknown> = {
  argumentos_inicio: { accion: 'asistente', dryRun: false },
  info_paquete: info,
  detectar: deteccion,
  juego_abierto: false,
  buscar_actualizacion: actualizacion,
}

export async function invocarSimulado<T>(comando: string): Promise<T> {
  await new Promise((r) => setTimeout(r, 50))
  if (comando === 'instalar') return new Promise<T>(() => {}) // se queda «instalando»
  if (comando in respuestas) return respuestas[comando] as T
  throw `Simulación: «${comando}» no disponible en el navegador.`
}

export const pantallaSimulada = p.get('pantalla')
