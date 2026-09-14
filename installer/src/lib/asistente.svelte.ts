// Estado compartido del asistente: qué pantalla se ve y lo que el usuario va eligiendo.
import {
  api,
  type Deteccion,
  type InfoActualizacion,
  type InfoPaquete,
  type Opciones,
  type ResultadoInstalacion,
  type ValidacionDatos,
  type ValidacionJuego,
} from './api'

export type Pantalla =
  | 'cargando'
  | 'bienvenida'
  | 'deteccion'
  | 'noEncontrado'
  | 'opciones'
  | 'resumen'
  | 'instalando'
  | 'listo'
  | 'mantenimiento'

export const PASOS = ['Inicio', 'Juego', 'Opciones', 'Instalación', 'Listo']

export function indicePaso(p: Pantalla): number {
  switch (p) {
    case 'deteccion':
    case 'noEncontrado':
      return 1
    case 'opciones':
      return 2
    case 'resumen':
    case 'instalando':
      return 3
    case 'listo':
      return 4
    default:
      return 0
  }
}

type Modo = 'auto' | 'manual'

export const asistente = $state({
  pantalla: 'cargando' as Pantalla,
  info: null as InfoPaquete | null,
  deteccion: null as Deteccion | null,
  errorCarga: '',

  // Juego
  modoJuego: 'auto' as Modo,
  rutaJuegoManual: '',
  validacionJuegoManual: null as ValidacionJuego | null,
  /** «Instalar igualmente» sin juego detectado. */
  sinJuego: false,
  aceptarVersion: false,

  // Carpeta del parche
  modoDatos: 'auto' as Modo,
  rutaDatosManual: '',
  validacionDatosManual: null as ValidacionDatos | null,

  // Opciones
  copiaSeguridad: true,
  autoActualizar: true,
  intervalo: 24,
  accesoDirecto: true,

  // Resultado
  resultado: null as ResultadoInstalacion | null,

  // Actualizaciones encontradas en segundo plano
  actualizacion: null as InfoActualizacion | null,
})

export async function cargarDeteccion() {
  asistente.errorCarga = ''
  try {
    const [info, deteccion] = await Promise.all([api.infoPaquete(), api.detectar()])
    asistente.info = info
    asistente.deteccion = deteccion
    asistente.intervalo = deteccion.estado?.updateIntervalHours ?? info.intervaloPorDefecto
    if (deteccion.estado) {
      asistente.autoActualizar = deteccion.estado.autoUpdate
      asistente.accesoDirecto = deteccion.estado.shortcut
    }
  } catch (e) {
    asistente.errorCarga = String(e)
  }
}

/** Busca actualizaciones sin molestar si falla (sin internet, sin release publicada…). */
export async function buscarEnSegundoPlano() {
  try {
    asistente.actualizacion = await api.buscarActualizacion()
  } catch {
    asistente.actualizacion = null
  }
}

export function validacionJuegoActual(): ValidacionJuego | null {
  if (asistente.modoJuego === 'manual') return asistente.validacionJuegoManual
  return asistente.deteccion?.validacionJuego ?? null
}

export function validacionDatosActual(): ValidacionDatos | null {
  if (asistente.modoDatos === 'manual') return asistente.validacionDatosManual
  return asistente.deteccion?.validacionDatos ?? null
}

export function juegoListo(): boolean {
  if (asistente.sinJuego) return true
  const v = validacionJuegoActual()
  if (!v?.ok) return false
  return v.versionCompatible !== false || asistente.aceptarVersion
}

export function opcionesInstalacion(): Opciones | null {
  const datos = validacionDatosActual()
  if (!datos?.ok) return null
  const juego = asistente.sinJuego ? null : validacionJuegoActual()
  return {
    rutaDatos: datos.rutaNormalizada,
    modoRuta: asistente.modoDatos,
    rutaJuego: juego?.ok ? juego.ruta : null,
    copiaSeguridad: datos.estado?.tipo === 'ajenos' ? asistente.copiaSeguridad : true,
    autoActualizar: asistente.autoActualizar,
    intervaloActualizacionHoras: asistente.intervalo,
    accesoDirecto: asistente.accesoDirecto,
    aceptarVersionNoProbada: asistente.aceptarVersion,
  }
}
