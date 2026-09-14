<script lang="ts">
  import Casilla from '../componentes/Casilla.svelte'
  import SelectorRuta from '../componentes/SelectorRuta.svelte'
  import Tarjeta from '../componentes/Tarjeta.svelte'
  import { api, formatoMB } from '../lib/api'
  import {
    asistente,
    cargarDeteccion,
    juegoListo,
    validacionDatosActual,
    validacionJuegoActual,
  } from '../lib/asistente.svelte'

  let buscando = $state(false)

  const d = $derived(asistente.deteccion)
  const vJuego = $derived(validacionJuegoActual())
  const vDatos = $derived(validacionDatosActual())
  const puedeSeguir = $derived(juegoListo() && !!vDatos?.ok)

  const estadoJuego = $derived(
    asistente.sinJuego ? 'aviso' : !vJuego ? 'aviso' : !vJuego.ok ? 'error' : vJuego.versionCompatible === false ? 'aviso' : 'ok',
  )
  const estadoDatos = $derived(!vDatos ? 'error' : !vDatos.ok ? 'error' : vDatos.estado?.tipo === 'ajenos' ? 'aviso' : 'ok')

  async function volverABuscar() {
    buscando = true
    await cargarDeteccion()
    buscando = false
    if (!asistente.deteccion?.juego && asistente.modoJuego === 'auto' && !asistente.sinJuego) {
      asistente.pantalla = 'noEncontrado'
    }
  }

  function describirCarpeta() {
    const e = vDatos?.estado
    if (!vDatos || !e) return ''
    if (e.tipo === 'vacia')
      return vDatos.existe ? 'Lista para instalar.' : 'Lista para instalar. La carpeta se creará al instalar (es normal).'
    if (e.tipo === 'parche')
      return `Ya tiene el parche ${e.versiones.join(', ')}${e.completo ? '' : ' (incompleto)'}: se actualizará encima.`
    return `Hay ${e.archivos.length} archivos .epk que no son del parche (otro mod o una traducción distinta).`
  }
</script>

<div class="pantalla">
  <header>
    <h2>¿Dónde está el juego?</h2>
    <p class="suave">Lo buscamos solos. Si tienes el juego en otro sitio, elige la carpeta a mano.</p>
  </header>

  <div class="tarjetas">
    <Tarjeta titulo="Juego" estado={estadoJuego}>
      {#if asistente.sinJuego}
        <p>Se instalará sin comprobar el juego.</p>
        <p class="suave">El parche funciona igual; solo no podremos comprobar la versión ni abrirlo desde aquí.</p>
        <button class="enlace" onclick={() => (asistente.sinJuego = false)}>Buscar el juego de nuevo</button>
      {:else}
        <SelectorRuta
          etiqueta="Carpeta del juego"
          bind:modo={asistente.modoJuego}
          bind:rutaManual={asistente.rutaJuegoManual}
          bind:validacion={asistente.validacionJuegoManual}
          validar={api.validarJuego}
          tituloDialogo="Elige la carpeta donde está GALAXYZ neo"
          rutaAuto={d?.juego?.ruta ?? null}
        />
        {#if asistente.modoJuego === 'auto' && d?.juego}
          <p class="detalle">
            <strong>{d.juego.nombre}</strong>{#if d.juego.version}&nbsp;· versión {d.juego.version}{/if}
            {#if d.juego.origen === 'steam'}&nbsp;· Steam{/if}
          </p>
        {:else if asistente.modoJuego === 'manual' && vJuego?.ok && vJuego.version}
          <p class="detalle">Versión {vJuego.version}</p>
        {/if}
        {#each vJuego?.avisos ?? [] as aviso}<p class="aviso">{aviso}</p>{/each}
        {#if vJuego?.ok && vJuego.versionCompatible === false}
          <div class="confirmar">
            <Casilla bind:marcada={asistente.aceptarVersion} titulo="Instalar igualmente">
              Puede que haya textos raros o que el juego se cierre. Siempre puedes desinstalar.
            </Casilla>
          </div>
        {/if}
      {/if}
    </Tarjeta>

    <Tarjeta titulo="Carpeta del parche" estado={estadoDatos}>
      <SelectorRuta
        etiqueta="Carpeta de datos"
        bind:modo={asistente.modoDatos}
        bind:rutaManual={asistente.rutaDatosManual}
        bind:validacion={asistente.validacionDatosManual}
        validar={api.validarDatos}
        tituloDialogo="Elige la carpeta de datos del juego (…\fuzz\galaxyz\data)"
        rutaAuto={d?.validacionDatos?.rutaNormalizada ?? null}
      />
      {#if vDatos?.ok}
        {#if asistente.modoDatos === 'manual'}<p class="ruta">{vDatos.rutaNormalizada}</p>{/if}
        <p class="detalle">{describirCarpeta()}</p>
        <p class="suave">
          Ocupará {formatoMB(vDatos.espacioNecesario)}{#if vDatos.espacioLibre !== null}&nbsp;· libres {formatoMB(vDatos.espacioLibre)}{/if}
        </p>
      {/if}
      {#if asistente.modoDatos === 'auto'}
        {#each vDatos?.errores ?? [] as e}<p class="error">{e}</p>{/each}
      {/if}
      {#each vDatos?.avisos ?? [] as a}<p class="aviso">{a}</p>{/each}
    </Tarjeta>
  </div>

  <footer>
    <button class="boton" onclick={() => (asistente.pantalla = 'bienvenida')}>← Atrás</button>
    <button class="boton" onclick={volverABuscar} disabled={buscando}>{buscando ? 'Buscando…' : 'Volver a buscar'}</button>
    <span class="espacio"></span>
    <button class="boton boton-principal" disabled={!puedeSeguir} onclick={() => (asistente.pantalla = 'opciones')}>
      Siguiente →
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

  .tarjetas {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
    align-items: start;
  }

  .suave {
    color: var(--texto-suave);
    font-size: 12.5px;
  }

  .detalle {
    margin-top: 8px;
    font-size: 13px;
  }

  .ruta {
    margin-top: 6px;
    font-family: 'Cascadia Mono', Consolas, monospace;
    font-size: 11.5px;
    color: var(--lavanda);
    word-break: break-all;
  }

  .aviso {
    color: var(--ambar);
    font-size: 12.5px;
    margin-top: 6px;
  }

  .error {
    color: var(--rojo);
    font-size: 12.5px;
    margin-top: 6px;
  }

  .confirmar {
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--lavanda-borde);
  }

  .enlace {
    margin-top: 8px;
    border: 0;
    background: none;
    color: var(--cian);
    padding: 0;
    cursor: pointer;
    font-size: 12.5px;
  }

  footer {
    margin-top: auto;
    display: flex;
    gap: 10px;
    align-items: center;
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
