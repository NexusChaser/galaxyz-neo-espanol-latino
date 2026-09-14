<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import personajes from '../assets/chara.webp'
  import AnilloProgreso from '../componentes/AnilloProgreso.svelte'
  import { api, mensajeError } from '../lib/api'
  import { asistente } from '../lib/asistente.svelte'

  let error = $state('')
  const juego = $derived(asistente.deteccion?.juego ?? null)
  const avisos = $derived(asistente.resultado?.avisos ?? [])

  async function abrirJuego() {
    error = ''
    try {
      await api.abrirJuego(juego?.appId ?? null, juego?.ruta ?? null)
    } catch (e) {
      error = mensajeError(e)
    }
  }
</script>

<div class="pantalla">
  <div class="texto">
    <div class="cabecera">
      <AnilloProgreso porcentaje={100} estado="ok" tamano={84} />
      <div>
        <h2>¡Parche instalado!</h2>
        <p class="suave">
          {asistente.resultado?.archivosEscritos ?? 0} archivos escritos{#if asistente.resultado?.archivosSinCambios}, {asistente.resultado.archivosSinCambios} ya estaban al día{/if}.
        </p>
      </div>
    </div>

    <div class="aviso ambar">
      <strong>Es normal que el juego se cierre mientras carga la pantalla de inicio.</strong>
      Sigue abriéndolo hasta que entre: sí funciona. Pasa más la primera vez y después de reiniciar la PC.
    </div>

    <div class="aviso">
      🖼️ Los textos que forman parte de imágenes todavía no están traducidos; llegarán en futuras actualizaciones.
    </div>

    {#if !asistente.autoActualizar}
      <div class="aviso lavanda">
        La actualización automática está desactivada. Puedes activarla cuando quieras desde el menú Inicio →
        <strong>Parche GALAXYZ neo → Cambiar opciones</strong>.
      </div>
    {/if}

    {#each avisos as a}<p class="texto-ambar">{a}</p>{/each}
    {#if error}<p class="texto-error">{error}</p>{/if}

    <div class="acciones">
      {#if juego}
        <button class="boton boton-principal" onclick={abrirJuego}>Abrir el juego</button>
      {/if}
      <button class="boton" onclick={() => asistente.info && api.abrirUrl(asistente.info.urlReportar)}>Reportar un error</button>
      <button class="boton" onclick={() => getCurrentWindow().close()}>Cerrar</button>
    </div>
    <p class="fans">Proyecto de fans, para fans · Sin relación con fuzz ni con Rensuke Oshikiri.</p>
  </div>
  <img class="personajes" src={personajes} alt="" aria-hidden="true" />
</div>

<style>
  .pantalla {
    position: relative;
    height: 100%;
    padding: 0 40px 20px;
    animation: entrar 400ms ease both;
  }

  .texto {
    position: relative;
    z-index: 2;
    width: 560px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .cabecera {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  h2 {
    font-size: 26px;
  }

  .suave {
    color: var(--texto-suave);
    font-size: 12.5px;
  }

  .aviso {
    padding: 10px 14px;
    border-radius: 12px;
    background: var(--morado-vidrio);
    border: 1px solid var(--lavanda-borde);
    backdrop-filter: var(--desenfoque);
    font-size: 13px;
  }

  .ambar {
    border-color: rgba(255, 179, 71, 0.45);
  }

  .ambar strong {
    color: var(--ambar);
    display: block;
  }

  .lavanda {
    color: var(--lavanda);
  }

  .texto-ambar {
    color: var(--ambar);
    font-size: 12.5px;
  }

  .texto-error {
    color: var(--rojo);
    font-size: 12.5px;
  }

  .acciones {
    margin-top: 4px;
    display: flex;
    gap: 10px;
  }

  .fans {
    font-size: 11px;
    color: var(--texto-suave);
    opacity: 0.75;
  }

  .personajes {
    position: absolute;
    right: -60px;
    bottom: -60px;
    height: 470px;
    z-index: 1;
    opacity: 0.95;
    mask-image: linear-gradient(to bottom, #000 70%, transparent 100%);
    animation: deslizar 800ms 100ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
  }

  @keyframes entrar {
    from {
      opacity: 0;
    }
  }

  @keyframes deslizar {
    from {
      opacity: 0;
      transform: translateX(60px);
    }
  }
</style>
