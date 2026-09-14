<script lang="ts">
  import type { Snippet } from 'svelte'

  let {
    marcada = $bindable(false),
    titulo,
    deshabilitada = false,
    children,
  }: { marcada?: boolean; titulo: string; deshabilitada?: boolean; children?: Snippet } = $props()
</script>

<label class="casilla" class:deshabilitada>
  <input type="checkbox" bind:checked={marcada} disabled={deshabilitada} />
  <span class="caja" aria-hidden="true">
    <svg viewBox="0 0 16 16"><path d="M3.5 8.5l3 3 6-7" /></svg>
  </span>
  <span class="texto">
    <span class="titulo">{titulo}</span>
    {#if children}<span class="detalle">{@render children()}</span>{/if}
  </span>
</label>

<style>
  .casilla {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    cursor: pointer;
  }

  .deshabilitada {
    opacity: 0.5;
    cursor: not-allowed;
  }

  input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .caja {
    flex: none;
    width: 20px;
    height: 20px;
    margin-top: 1px;
    border-radius: 6px;
    border: 1.5px solid var(--lavanda-borde);
    background: rgba(255, 255, 255, 0.04);
    display: grid;
    place-items: center;
    transition:
      background var(--transicion),
      border-color var(--transicion),
      box-shadow var(--transicion);
  }

  svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: #fff;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-dasharray: 16;
    stroke-dashoffset: 16;
    transition: stroke-dashoffset var(--transicion);
  }

  input:checked + .caja {
    background: var(--rosa);
    border-color: transparent;
    box-shadow: 0 0 12px var(--rosa-brillo);
  }

  input:checked + .caja svg {
    stroke-dashoffset: 0;
  }

  input:focus-visible + .caja {
    outline: 2px solid var(--cian);
    outline-offset: 2px;
  }

  .texto {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .titulo {
    font-weight: 600;
  }

  .detalle {
    color: var(--texto-suave);
    font-size: 12.5px;
  }
</style>
