<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { onDestroy, onMount } from 'svelte'
  import logo from '../assets/logo.webp'
  import Casilla from '../componentes/Casilla.svelte'
  import Dialogo from '../componentes/Dialogo.svelte'
  import PanelProgreso from '../componentes/PanelProgreso.svelte'
  import SelectorFrecuencia from '../componentes/SelectorFrecuencia.svelte'
  import Tarjeta from '../componentes/Tarjeta.svelte'
  import { api, formatoFecha, formatoMB, mensajeError, nombreIntervalo, type Progreso } from '../lib/api'
  import { asistente, cargarDeteccion } from '../lib/asistente.svelte'

  let {
    accionInicial = null,
    alConsumirAccion,
  }: { accionInicial?: 'desinstalar' | 'reparar' | 'actualizar' | null; alConsumirAccion?: () => void } = $props()

  type Operacion = 'buscar' | 'actualizar' | 'reparar' | 'desinstalar' | 'instalador'
  const TITULOS: Record<Operacion, string> = {
    buscar: 'Buscando actualizaciones…',
    actualizar: 'Actualizando la traducción…',
    reparar: 'Reparando el parche…',
    desinstalar: 'Desinstalando el parche…',
    instalador: 'Actualizando el instalador…',
  }

  let operacion = $state<Operacion | null>(null)
  let progreso = $state<Progreso | null>(null)
  let registro = $state<string[]>([])
  let exito = $state('')
  let error = $state('')
  let desinstalado = $state(false)
  let ocultarAvisoJuego = $state(false)

  let dialogoOpciones = $state(false)
  let dialogoDesinstalar = $state(false)
  let borrarCopia = $state(false)
  let autoActualizar = $state(false)
  let intervalo = $state(24)

  const d = $derived(asistente.deteccion)
  const e = $derived(d?.estado ?? null)
  const act = $derived(asistente.actualizacion)
  const juegoNoSoportado = $derived(
    !ocultarAvisoJuego && (e?.unsupportedGameVersion || (act?.compatibleConJuego === false ? act.versionJuego : null)),
  )

  let quitar: (() => void) | undefined
  onDestroy(() => quitar?.())

  async function ejecutar(op: Operacion, trabajo: () => Promise<string>) {
    operacion = op
    progreso = null
    registro = []
    exito = ''
    error = ''
    quitar?.()
    quitar = await api.alProgresar((p) => {
      progreso = p
      if (registro.at(-1) !== p.mensaje) registro = [...registro.slice(-200), p.mensaje]
    })
    try {
      exito = await trabajo()
    } catch (err) {
      error = mensajeError(err)
    } finally {
      quitar?.()
      quitar = undefined
      await cargarDeteccion()
      operacion = null
    }
  }

  const buscar = () =>
    ejecutar('buscar', async () => {
      progreso = { paso: 'comprobando', porcentaje: 30, mensaje: 'Consultando GitHub…' }
      const i = await api.buscarActualizacion()
      asistente.actualizacion = i
      if (i.hayParcheNuevo) return `Hay una versión nueva de la traducción: ${i.versionDisponible}.`
      if (i.compatibleConJuego === false) return 'El parche está al día, pero todavía no soporta tu versión del juego.'
      return `El parche está al día (versión ${i.versionInstalada ?? i.versionDisponible}).`
    })

  const actualizar = () =>
    ejecutar('actualizar', async () => {
      const r = await api.aplicarActualizacion()
      asistente.actualizacion = asistente.actualizacion && { ...asistente.actualizacion, hayParcheNuevo: false }
      return `Traducción actualizada a la versión ${r.version}.`
    })

  const reparar = () =>
    ejecutar('reparar', async () => {
      const r = await api.reparar()
      return r.archivosEscritos === 0
        ? 'Todo estaba bien: no hizo falta cambiar ningún archivo.'
        : `Reparado: se volvieron a copiar ${r.archivosEscritos} archivos.`
    })

  const desinstalar = () => {
    dialogoDesinstalar = false
    return ejecutar('desinstalar', async () => {
      const r = await api.desinstalar(borrarCopia)
      desinstalado = true
      const partes = [`Se quitaron ${r.borrados.length} archivos del parche.`]
      if (r.restaurados.length) partes.push(`Se devolvieron ${r.restaurados.length} archivos de la copia de seguridad.`)
      if (r.conservados.length) partes.push(`${r.conservados.length} archivos modificados a mano no se tocaron.`)
      return partes.join(' ')
    })
  }

  const actualizarInstalador = () =>
    ejecutar('instalador', async () => `Instalador actualizado a ${await api.actualizarInstalador()}. Se reiniciará solo.`)

  function abrirOpciones() {
    autoActualizar = e?.autoUpdate ?? false
    intervalo = e?.updateIntervalHours ?? 24
    dialogoOpciones = true
  }

  async function guardarOpciones() {
    dialogoOpciones = false
    const activar = autoActualizar
    await ejecutar('buscar', async () => {
      await api.cambiarOpciones(activar, intervalo)
      if (!activar) return 'Actualización automática desactivada.'
      // Al activarla se hace una primera comprobación en ese momento.
      progreso = { paso: 'comprobando', porcentaje: 50, mensaje: 'Primera comprobación…' }
      try {
        asistente.actualizacion = await api.buscarActualizacion()
        const extra = asistente.actualizacion.hayParcheNuevo ? ` Hay una versión nueva: ${asistente.actualizacion.versionDisponible}.` : ''
        return `Actualización automática activada: ${nombreIntervalo(intervalo).replace(' (recomendado)', '').toLowerCase()}.${extra}`
      } catch (err) {
        return `Actualización automática activada. La primera comprobación falló: ${mensajeError(err)}`
      }
    })
  }

  // Acciones pedidas desde la línea de comandos o «Aplicaciones instaladas»: solo al abrir.
  onMount(() => {
    const accion = accionInicial
    alConsumirAccion?.()
    if (accion === 'desinstalar') dialogoDesinstalar = true
    else if (accion === 'reparar') reparar()
    else if (accion === 'actualizar') buscar()
  })
</script>

<div class="pantalla">
  {#if operacion}
    <div class="centro">
      <PanelProgreso {progreso} {registro} titulo={TITULOS[operacion]} />
    </div>
  {:else if desinstalado}
    <div class="centro">
      <h2>Parche desinstalado</h2>
      <p class="suave espaciado">{exito}</p>
      <p class="suave">El juego vuelve a usar sus textos originales.</p>
      <div class="acciones centradas">
        <button class="boton boton-principal" onclick={() => { desinstalado = false; asistente.pantalla = 'bienvenida' }}>
          Volver a instalar
        </button>
        <button class="boton" onclick={() => getCurrentWindow().close()}>Cerrar</button>
      </div>
    </div>
  {:else}
    <header>
      <img src={logo} alt="GALAXYZ neo" />
      <div>
        <h2>Mantenimiento</h2>
        <p class="suave">Parche en Español Latino · proyecto de fans, para fans</p>
      </div>
    </header>

    <div class="cuerpo">
      <Tarjeta titulo="Instalación">
        {#if e}
          <dl>
            <dt>Parche</dt>
            <dd>versión {e.patchVersion} · instalado {formatoFecha(e.installedAt)}</dd>
            <dt>Juego</dt>
            <dd>{d?.juego ? `${d.juego.nombre} · ${d.juego.version ?? 'versión desconocida'}` : 'No detectado'}</dd>
            <dt>Carpeta</dt>
            <dd class="ruta">{e.dataPath}</dd>
            <dt>Actualización automática</dt>
            <dd>
              {#if e.autoUpdate}
                <span class="cian">Activada</span> · {nombreIntervalo(e.updateIntervalHours).replace(' (recomendado)', '').toLowerCase()}
              {:else}
                <span class="ambar">Desactivada</span>
              {/if}
            </dd>
            <dt>Última comprobación</dt>
            <dd>{formatoFecha(e.lastCheck)}{#if e.lastCheckResult}<span class="suave"> · {e.lastCheckResult}</span>{/if}</dd>
          </dl>
        {:else}
          <p>No hay ninguna instalación registrada.</p>
        {/if}
      </Tarjeta>

      <div class="avisos">
        {#if exito}<div class="banda cian-borde">{exito}</div>{/if}
        {#if error}<div class="banda rojo-borde">{error}</div>{/if}

        {#if act?.hayParcheNuevo}
          <div class="banda rosa-borde">
            <strong>Hay una versión nueva de la traducción: {act.versionDisponible}</strong>
            {#if act.notas}<p class="suave">{act.notas}</p>{/if}
            <div class="acciones"><button class="boton boton-principal chico" onclick={actualizar}>Actualizar ahora{act.tamanoDescarga ? ` (${formatoMB(act.tamanoDescarga)})` : ''}</button></div>
          </div>
        {/if}

        {#if juegoNoSoportado}
          <div class="banda ambar-borde">
            <strong>El juego se actualizó (versión {juegoNoSoportado}) y el parche todavía no la soporta.</strong>
            <p class="suave">Puede haber textos raros o cierres. Puedes quitar el parche hasta que salga una versión compatible.</p>
            <div class="acciones">
              <button class="boton chico" onclick={() => (dialogoDesinstalar = true)}>Quitar el parche por ahora</button>
              <button class="boton chico" onclick={() => (ocultarAvisoJuego = true)}>Mantenerlo</button>
            </div>
          </div>
        {/if}

        {#if (d?.archivosDanados.length ?? 0) > 0}
          <div class="banda ambar-borde">
            <strong>
              {d?.archivosDanados.length === 1
                ? 'Falta o está dañado 1 archivo del parche.'
                : `Faltan o están dañados ${d?.archivosDanados.length} archivos del parche.`}
            </strong>
            <div class="acciones"><button class="boton chico" onclick={reparar}>Reparar</button></div>
          </div>
        {/if}

        {#if act?.hayInstaladorNuevo}
          <div class="banda lavanda-borde">
            <strong>Hay una versión nueva del instalador: {act.instaladorDisponible}</strong>
            <div class="acciones"><button class="boton chico" onclick={actualizarInstalador}>Actualizar instalador</button></div>
          </div>
        {/if}

        {#if e && !e.autoUpdate}
          <p class="suave">
            La actualización automática está desactivada. Actívala en <strong>Cambiar opciones</strong> o busca a mano cuando quieras.
          </p>
        {/if}
      </div>
    </div>

    <footer>
      <button class="boton" onclick={buscar}>Buscar actualizaciones</button>
      <button class="boton" onclick={reparar} disabled={!e}>Reparar</button>
      <button class="boton" onclick={abrirOpciones} disabled={!e}>Cambiar opciones</button>
      <span class="espacio"></span>
      <button class="boton peligro" onclick={() => (dialogoDesinstalar = true)} disabled={!e}>Desinstalar</button>
    </footer>
  {/if}
</div>

<Dialogo titulo="Cambiar opciones" bind:abierto={dialogoOpciones}>
  <Casilla bind:marcada={autoActualizar} titulo="Mantener el parche actualizado automáticamente">
    Busca traducciones nuevas y las instala solas cuando el juego está cerrado.
  </Casilla>
  {#if autoActualizar}
    <SelectorFrecuencia bind:horas={intervalo} opciones={asistente.info?.intervalos ?? [6, 12, 24, 168]} />
  {/if}
  {#snippet acciones()}
    <button class="boton" onclick={() => (dialogoOpciones = false)}>Cancelar</button>
    <button class="boton boton-principal" onclick={guardarOpciones}>Guardar</button>
  {/snippet}
</Dialogo>

<Dialogo titulo="¿Desinstalar el parche?" bind:abierto={dialogoDesinstalar}>
  <p>Se quitarán los archivos de la traducción y el juego volverá a estar en inglés. Tu partida no se toca.</p>
  {#if e?.backupPath}
    <Casilla bind:marcada={borrarCopia} titulo="Borrar también la copia de seguridad">
      Primero se devuelven sus archivos; después se borra la carpeta.
    </Casilla>
  {/if}
  {#snippet acciones()}
    <button class="boton" onclick={() => (dialogoDesinstalar = false)}>Cancelar</button>
    <button class="boton peligro-lleno" onclick={desinstalar}>Desinstalar</button>
  {/snippet}
</Dialogo>

<style>
  .pantalla {
    height: 100%;
    padding: 0 40px 22px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    animation: entrar 360ms ease both;
  }

  .centro {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  header {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  header img {
    width: 170px;
    filter: drop-shadow(0 0 14px rgba(190, 120, 255, 0.5));
  }

  h2 {
    font-size: 24px;
  }

  .cuerpo {
    display: grid;
    grid-template-columns: 1.05fr 1fr;
    gap: 14px;
    min-height: 0;
    flex: 1;
    align-items: start;
  }

  dl {
    margin: 0;
    display: grid;
    grid-template-columns: 150px 1fr;
    row-gap: 8px;
    column-gap: 12px;
    font-size: 12.5px;
  }

  dt {
    color: var(--texto-suave);
  }

  dd {
    margin: 0;
  }

  .ruta {
    font-family: 'Cascadia Mono', Consolas, monospace;
    font-size: 11.5px;
    color: var(--lavanda);
    word-break: break-all;
  }

  .avisos {
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow-y: auto;
    padding-right: 4px;
  }

  .banda {
    padding: 10px 14px;
    border-radius: 12px;
    background: var(--morado-vidrio);
    backdrop-filter: var(--desenfoque);
    border: 1px solid var(--lavanda-borde);
    font-size: 13px;
  }

  .rosa-borde {
    border-color: var(--rosa);
    box-shadow: 0 0 20px rgba(255, 79, 150, 0.2);
  }

  .ambar-borde {
    border-color: rgba(255, 179, 71, 0.5);
  }

  .ambar-borde strong {
    color: var(--ambar);
  }

  .cian-borde {
    border-color: rgba(127, 231, 255, 0.5);
  }

  .rojo-borde {
    border-color: rgba(255, 107, 122, 0.55);
    color: var(--rojo);
    white-space: pre-line;
  }

  .acciones {
    margin-top: 8px;
    display: flex;
    gap: 8px;
  }

  .centradas {
    margin-top: 20px;
    justify-content: center;
  }

  .chico {
    padding: 6px 14px;
    font-size: 12.5px;
  }

  .suave {
    color: var(--texto-suave);
    font-size: 12.5px;
  }

  .espaciado {
    margin-top: 8px;
    max-width: 520px;
  }

  .cian {
    color: var(--cian);
  }

  .ambar {
    color: var(--ambar);
  }

  footer {
    display: flex;
    gap: 10px;
  }

  .espacio {
    flex: 1;
  }

  .peligro:hover {
    border-color: var(--rojo);
    color: var(--rojo);
  }

  .peligro-lleno {
    background: #d9364a;
    border-color: transparent;
    color: #fff;
  }

  @keyframes entrar {
    from {
      opacity: 0;
    }
  }
</style>
