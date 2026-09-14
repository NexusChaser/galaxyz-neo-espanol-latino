<script lang="ts">
  import { onDestroy } from 'svelte'
  import PanelProgreso from '../componentes/PanelProgreso.svelte'
  import Tarjeta from '../componentes/Tarjeta.svelte'
  import { api, mensajeError, type Progreso } from '../lib/api'
  import { asistente, cargarDeteccion, opcionesInstalacion } from '../lib/asistente.svelte'

  let progreso = $state<Progreso | null>(null)
  let registro = $state<string[]>([])
  let error = $state('')
  let trabajando = $state(false)
  let quitar: (() => void) | undefined

  async function instalar() {
    const opciones = opcionesInstalacion()
    if (!opciones) {
      error = 'Faltan datos de la instalación. Vuelve atrás y revisa las rutas.'
      return
    }
    error = ''
    registro = []
    progreso = null
    trabajando = true
    quitar?.()
    quitar = await api.alProgresar((p) => {
      progreso = p
      if (registro.at(-1) !== p.mensaje) registro = [...registro.slice(-200), p.mensaje]
    })
    try {
      asistente.resultado = await api.instalar(opciones)
      await cargarDeteccion()
      asistente.pantalla = 'listo'
    } catch (e) {
      error = mensajeError(e)
    } finally {
      trabajando = false
    }
  }

  instalar()
  onDestroy(() => quitar?.())
</script>

<div class="pantalla">
  <PanelProgreso
    {progreso}
    {registro}
    estado={error ? 'error' : 'activo'}
    titulo={error ? 'No se pudo instalar' : 'Instalando el parche…'}
  />

  {#if error}
    <div class="error">
      <Tarjeta titulo="Qué pasó" estado="error">
        <p class="texto-error">{error}</p>
        <p class="suave">No se dejó nada a medias: la carpeta del juego quedó como estaba.</p>
      </Tarjeta>
      <div class="acciones">
        <button class="boton" onclick={() => (asistente.pantalla = 'resumen')}>← Atrás</button>
        <button class="boton boton-principal" onclick={instalar} disabled={trabajando}>Reintentar</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .pantalla {
    height: 100%;
    padding: 6px 40px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    animation: entrar 300ms ease both;
  }

  .error {
    margin-top: 12px;
    width: 560px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .texto-error {
    white-space: pre-line;
  }

  .suave {
    margin-top: 6px;
    color: var(--texto-suave);
    font-size: 12.5px;
  }

  .acciones {
    display: flex;
    justify-content: space-between;
  }

  @keyframes entrar {
    from {
      opacity: 0;
    }
  }
</style>
