<script lang="ts">
  import BarraTitulo from './componentes/BarraTitulo.svelte'
  import Fondo from './componentes/Fondo.svelte'
  import IndicadorPasos from './componentes/IndicadorPasos.svelte'
  import Tarjeta from './componentes/Tarjeta.svelte'
  import Bienvenida from './pasos/Bienvenida.svelte'
  import Deteccion from './pasos/Deteccion.svelte'
  import Instalando from './pasos/Instalando.svelte'
  import JuegoNoEncontrado from './pasos/JuegoNoEncontrado.svelte'
  import Listo from './pasos/Listo.svelte'
  import Mantenimiento from './pasos/Mantenimiento.svelte'
  import Opciones from './pasos/Opciones.svelte'
  import Resumen from './pasos/Resumen.svelte'
  import { api, enTauri, type ArgumentosInicio } from './lib/api'
  import { pantallaSimulada } from './lib/simulacion'
  import { asistente, buscarEnSegundoPlano, cargarDeteccion, indicePaso, PASOS } from './lib/asistente.svelte'

  let accionInicial = $state<'desinstalar' | 'reparar' | 'actualizar' | null>(null)

  async function iniciar() {
    if (!enTauri()) {
      if (!(import.meta.env.DEV || import.meta.env.VITE_SIMULACION === '1')) return
      // Vista previa en el navegador con datos de ejemplo (src/lib/simulacion.ts).
      await cargarDeteccion()
      await buscarEnSegundoPlano()
      asistente.pantalla = (pantallaSimulada as typeof asistente.pantalla) ?? 'bienvenida'
      return
    }
    let args: ArgumentosInicio | null = null
    try {
      args = await api.argumentosInicio()
    } catch {
      args = null
    }
    await cargarDeteccion()
    if (args && ['desinstalar', 'reparar', 'actualizar'].includes(args.accion)) {
      accionInicial = args.accion as typeof accionInicial
    }
    asistente.pantalla = asistente.deteccion?.estado || accionInicial ? 'mantenimiento' : 'bienvenida'
    buscarEnSegundoPlano()
  }

  function comenzar() {
    const d = asistente.deteccion
    asistente.pantalla = !d?.juego && asistente.modoJuego === 'auto' && !asistente.sinJuego ? 'noEncontrado' : 'deteccion'
  }

  iniciar()

  const paso = $derived(indicePaso(asistente.pantalla))
  const conPasos = $derived(paso > 0 && asistente.pantalla !== 'mantenimiento')
</script>

<div class="ventana">
  <Fondo />
  <BarraTitulo version={asistente.info?.version ?? ''} />

  {#if conPasos}
    <nav class="navegacion">
      <IndicadorPasos pasos={PASOS} actual={paso} />
    </nav>
  {/if}

  <main class="contenido">
    {#if asistente.errorCarga}
      <div class="error-carga">
        <Tarjeta titulo="No se pudo analizar la PC" estado="error">
          <p>{asistente.errorCarga}</p>
          <button class="boton" onclick={iniciar}>Reintentar</button>
        </Tarjeta>
      </div>
    {/if}
    {#key asistente.pantalla}
      {#if asistente.pantalla === 'cargando'}
        <div class="cargando"><span class="punto"></span> Analizando la PC…</div>
      {:else if asistente.pantalla === 'bienvenida'}
        <Bienvenida
          version={asistente.info?.version ?? ''}
          reinstalacion={!!asistente.deteccion?.estado}
          alComenzar={comenzar}
        />
      {:else if asistente.pantalla === 'deteccion'}
        <Deteccion />
      {:else if asistente.pantalla === 'noEncontrado'}
        <JuegoNoEncontrado />
      {:else if asistente.pantalla === 'opciones'}
        <Opciones />
      {:else if asistente.pantalla === 'resumen'}
        <Resumen />
      {:else if asistente.pantalla === 'instalando'}
        <Instalando />
      {:else if asistente.pantalla === 'listo'}
        <Listo />
      {:else if asistente.pantalla === 'mantenimiento'}
        <Mantenimiento {accionInicial} alConsumirAccion={() => (accionInicial = null)} />
      {/if}
    {/key}
  </main>
</div>

<style>
  .ventana {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .navegacion {
    position: relative;
    z-index: 5;
    padding: 0 40px 12px;
  }

  .contenido {
    position: relative;
    z-index: 2;
    flex: 1;
    min-height: 0;
  }

  .cargando {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--texto-suave);
  }

  .punto {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--rosa);
    box-shadow: 0 0 12px var(--rosa-brillo);
    animation: latido 1s ease-in-out infinite;
  }

  .error-carga {
    position: absolute;
    z-index: 20;
    top: 0;
    left: 40px;
    right: 40px;
  }

  @keyframes latido {
    50% {
      transform: scale(0.6);
      opacity: 0.5;
    }
  }
</style>
