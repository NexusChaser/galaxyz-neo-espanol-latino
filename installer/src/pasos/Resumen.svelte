<script lang="ts">
  import { onDestroy } from 'svelte'
  import Tarjeta from '../componentes/Tarjeta.svelte'
  import { api, formatoMB, nombreIntervalo } from '../lib/api'
  import { asistente, opcionesInstalacion, validacionDatosActual, validacionJuegoActual } from '../lib/asistente.svelte'

  const opciones = $derived(opcionesInstalacion())
  const vDatos = $derived(validacionDatosActual())
  const vJuego = $derived(asistente.sinJuego ? null : validacionJuegoActual())
  const ajenos = $derived(vDatos?.estado?.tipo === 'ajenos' ? vDatos.estado.archivos.length : 0)

  let juegoAbierto = $state(false)
  const revisar = async () => {
    try {
      juegoAbierto = await api.juegoAbierto()
    } catch {
      juegoAbierto = false
    }
  }
  revisar()
  const intervalo = setInterval(revisar, 2000)
  onDestroy(() => clearInterval(intervalo))
</script>

<div class="pantalla">
  <header>
    <h2>Todo listo para instalar</h2>
    <p class="suave">Revisa lo que se va a hacer.</p>
  </header>

  <Tarjeta>
    <dl>
      <dt>Juego</dt>
      <dd>
        {#if vJuego?.ok}
          <span class="ruta">{vJuego.ruta}</span>
          {#if vJuego.version}<span class="suave">· versión {vJuego.version}{vJuego.versionCompatible === false ? ' (no probada)' : ''}</span>{/if}
        {:else}
          Sin comprobar
        {/if}
      </dd>

      <dt>Carpeta del parche</dt>
      <dd><span class="ruta">{opciones?.rutaDatos}</span></dd>

      <dt>Archivos</dt>
      <dd>
        {asistente.info?.copias ?? '—'} archivos
        <span class="suave">· {formatoMB(asistente.info?.tamanoInstalado ?? 0)} · parche v{asistente.info?.version}</span>
      </dd>

      <dt>Copia de seguridad</dt>
      <dd>
        {#if ajenos === 0}No hace falta{:else if asistente.copiaSeguridad}Sí, de {ajenos} archivos{:else}<span class="ambar">No (hay {ajenos} archivos ajenos que se sustituirán)</span>{/if}
      </dd>

      <dt>Actualización automática</dt>
      <dd>
        {#if asistente.autoActualizar}
          Activada · {nombreIntervalo(asistente.intervalo).replace(' (recomendado)', '').toLowerCase()} y al iniciar Windows
        {:else}
          <span class="ambar">Desactivada</span> <span class="suave">· se puede activar después en Mantenimiento → Cambiar opciones</span>
        {/if}
      </dd>

      <dt>Acceso directo</dt>
      <dd>{asistente.accesoDirecto ? 'Sí, en el menú Inicio' : 'No'}</dd>
    </dl>
  </Tarjeta>

  {#if juegoAbierto}
    <Tarjeta titulo="El juego está abierto" estado="aviso">
      <p>Ciérralo para poder instalar. Esto se comprueba solo cada 2 segundos.</p>
    </Tarjeta>
  {/if}

  <footer>
    <button class="boton" onclick={() => (asistente.pantalla = 'opciones')}>← Atrás</button>
    <span class="espacio"></span>
    <button class="boton boton-principal" disabled={!opciones || juegoAbierto} onclick={() => (asistente.pantalla = 'instalando')}>
      Instalar
    </button>
  </footer>
</div>

<style>
  .pantalla {
    height: 100%;
    padding: 0 40px 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    animation: entrar 360ms ease both;
  }

  h2 {
    font-size: 24px;
  }

  dl {
    margin: 0;
    display: grid;
    grid-template-columns: 170px 1fr;
    row-gap: 10px;
    column-gap: 16px;
  }

  dt {
    color: var(--texto-suave);
    font-size: 12.5px;
  }

  dd {
    margin: 0;
    font-size: 13px;
  }

  .ruta {
    font-family: 'Cascadia Mono', Consolas, monospace;
    font-size: 12px;
    color: var(--lavanda);
    word-break: break-all;
  }

  .suave {
    color: var(--texto-suave);
    font-size: 12.5px;
  }

  .ambar {
    color: var(--ambar);
  }

  footer {
    margin-top: auto;
    display: flex;
    gap: 10px;
  }

  .espacio {
    flex: 1;
  }

  @keyframes entrar {
    from {
      opacity: 0;
      transform: translateX(24px);
    }
  }
</style>
