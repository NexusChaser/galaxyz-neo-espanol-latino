<script lang="ts">
  import { nombreIntervalo } from '../lib/api'

  let { horas = $bindable(24), opciones }: { horas?: number; opciones: number[] } = $props()
</script>

<div class="frecuencia">
  <span class="etiqueta">Buscar actualizaciones</span>
  <div class="opciones" role="radiogroup" aria-label="Frecuencia">
    {#each opciones as h}
      <button role="radio" aria-checked={horas === h} class:activa={horas === h} onclick={() => (horas = h)}>
        {nombreIntervalo(h).replace(' (recomendado)', '')}
        {#if h === 24}<span class="recomendado">recomendado</span>{/if}
      </button>
    {/each}
  </div>
  <p class="nota">También se comprueba al iniciar Windows. Si la PC estaba apagada, se hace en cuanto se encienda.</p>
</div>

<style>
  .frecuencia {
    margin-left: 32px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .etiqueta {
    font-size: 12px;
    color: var(--texto-suave);
  }

  .opciones {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }

  button {
    position: relative;
    text-align: left;
    padding: 8px 10px;
    border-radius: 10px;
    border: 1px solid var(--lavanda-borde);
    background: rgba(255, 255, 255, 0.04);
    font-size: 12.5px;
    cursor: pointer;
    transition:
      border-color var(--transicion),
      background var(--transicion);
  }

  button:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .activa {
    border-color: var(--rosa);
    background: rgba(255, 79, 150, 0.14);
  }

  .recomendado {
    display: block;
    font-size: 10.5px;
    color: var(--cian);
  }

  .nota {
    font-size: 11.5px;
    color: var(--texto-suave);
  }
</style>
