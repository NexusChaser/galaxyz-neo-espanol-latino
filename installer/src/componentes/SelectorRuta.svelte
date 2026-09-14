<script lang="ts">
  // Automático / Manual con validación en vivo mientras se escribe o al elegir la carpeta.
  import { open } from '@tauri-apps/plugin-dialog'

  type Resultado = { ok: boolean; errores: string[]; avisos: string[] }

  let {
    etiqueta,
    modo = $bindable<'auto' | 'manual'>('auto'),
    rutaManual = $bindable(''),
    validacion = $bindable<Resultado | null>(null),
    validar,
    tituloDialogo,
    rutaAuto,
  }: {
    etiqueta: string
    modo?: 'auto' | 'manual'
    rutaManual?: string
    validacion?: Resultado | null
    validar: (ruta: string) => Promise<Resultado>
    tituloDialogo: string
    rutaAuto: string | null
  } = $props()

  let comprobando = $state(false)
  let temporizador: ReturnType<typeof setTimeout> | undefined
  let peticion = 0

  $effect(() => {
    const ruta = rutaManual.trim()
    if (modo !== 'manual') return
    clearTimeout(temporizador)
    if (!ruta) {
      validacion = null
      return
    }
    comprobando = true
    const n = ++peticion
    temporizador = setTimeout(async () => {
      try {
        const r = await validar(ruta)
        if (n === peticion) validacion = r
      } catch (e) {
        if (n === peticion) validacion = { ok: false, errores: [String(e)], avisos: [] }
      } finally {
        if (n === peticion) comprobando = false
      }
    }, 350)
    return () => clearTimeout(temporizador)
  })

  async function elegir() {
    const elegida = await open({ directory: true, multiple: false, title: tituloDialogo })
    if (typeof elegida === 'string') {
      modo = 'manual'
      rutaManual = elegida
    }
  }
</script>

<div class="selector">
  <div class="fila">
    <span class="etiqueta">{etiqueta}</span>
    <div class="segmentado" role="radiogroup" aria-label={etiqueta}>
      <button role="radio" aria-checked={modo === 'auto'} class:activo={modo === 'auto'} onclick={() => (modo = 'auto')}>
        Automático
      </button>
      <button role="radio" aria-checked={modo === 'manual'} class:activo={modo === 'manual'} onclick={() => (modo = 'manual')}>
        Manual
      </button>
    </div>
  </div>

  {#if modo === 'auto'}
    <p class="ruta">{rutaAuto ?? 'No detectada'}</p>
  {:else}
    <div class="entrada">
      <input type="text" bind:value={rutaManual} placeholder="C:\…" spellcheck="false" />
      <button class="boton pequeno" onclick={elegir}>Elegir…</button>
    </div>
    {#if comprobando}
      <p class="estado">Comprobando…</p>
    {:else if validacion}
      {#if validacion.ok}<p class="estado ok">✓ Carpeta válida</p>{/if}
      {#each validacion.errores as e}<p class="estado error">{e}</p>{/each}
    {/if}
  {/if}
</div>

<style>
  .selector {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .fila {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .etiqueta {
    font-size: 12px;
    color: var(--texto-suave);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .segmentado {
    display: flex;
    padding: 2px;
    border-radius: 999px;
    border: 1px solid var(--lavanda-borde);
    background: rgba(0, 0, 0, 0.2);
  }

  .segmentado button {
    border: 0;
    background: transparent;
    padding: 3px 12px;
    border-radius: 999px;
    font-size: 12px;
    color: var(--texto-suave);
    cursor: pointer;
    transition:
      background var(--transicion),
      color var(--transicion);
  }

  .segmentado button.activo {
    background: rgba(255, 255, 255, 0.14);
    color: var(--texto);
  }

  .ruta {
    font-family: 'Cascadia Mono', Consolas, monospace;
    font-size: 12px;
    color: var(--lavanda);
    word-break: break-all;
    user-select: text;
  }

  .entrada {
    display: flex;
    gap: 8px;
  }

  input {
    flex: 1;
    min-width: 0;
    padding: 7px 12px;
    border-radius: 10px;
    border: 1px solid var(--lavanda-borde);
    background: rgba(0, 0, 0, 0.3);
    color: var(--texto);
    font-family: 'Cascadia Mono', Consolas, monospace;
    font-size: 12px;
    user-select: text;
  }

  input:focus {
    outline: none;
    border-color: var(--cian);
  }

  .pequeno {
    padding: 6px 14px;
    font-size: 12px;
  }

  .estado {
    font-size: 12px;
    color: var(--texto-suave);
  }

  .ok {
    color: var(--cian);
  }

  .error {
    color: var(--rojo);
  }
</style>
