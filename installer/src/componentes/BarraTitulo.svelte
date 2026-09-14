<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { enTauri } from '../lib/api'

  let { version = '' }: { version?: string } = $props()

  const ventana = enTauri() ? getCurrentWindow() : null
</script>

<header class="barra" data-tauri-drag-region>
  <span class="titulo" data-tauri-drag-region>
    Parche GALAXYZ neo{#if version}<span class="version">v{version}</span>{/if}
  </span>
  <div class="controles">
    <button class="control" aria-label="Minimizar" onclick={() => ventana?.minimize()}>
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 5h8" stroke="currentColor" stroke-width="1.2" /></svg>
    </button>
    <button class="control cerrar" aria-label="Cerrar" onclick={() => ventana?.close()}>
      <svg width="10" height="10" viewBox="0 0 10 10">
        <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2" />
      </svg>
    </button>
  </div>
</header>

<style>
  .barra {
    position: relative;
    z-index: 10;
    height: var(--alto-barra);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-left: 16px;
  }

  .titulo {
    font-size: 12px;
    color: var(--texto-suave);
    letter-spacing: 0.02em;
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .version {
    padding: 1px 7px;
    border-radius: 999px;
    border: 1px solid var(--lavanda-borde);
    font-size: 11px;
  }

  .controles {
    display: flex;
    height: 100%;
  }

  .control {
    width: 46px;
    height: 100%;
    border: 0;
    background: transparent;
    color: var(--texto-suave);
    display: grid;
    place-items: center;
    cursor: pointer;
    transition:
      background var(--transicion),
      color var(--transicion);
  }

  .control:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--texto);
  }

  .cerrar:hover {
    background: #e81123;
    color: #fff;
  }
</style>
