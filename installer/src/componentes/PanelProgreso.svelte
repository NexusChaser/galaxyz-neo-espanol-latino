<script lang="ts">
  // Anillo + mensaje + registro desplegable. Lo usan la instalación y Mantenimiento.
  import AnilloProgreso from './AnilloProgreso.svelte'
  import type { Progreso } from '../lib/api'

  let {
    progreso,
    registro,
    estado = 'activo',
    titulo,
  }: { progreso: Progreso | null; registro: string[]; estado?: 'activo' | 'ok' | 'error'; titulo: string } = $props()

  let detalles = $state(false)
</script>

<div class="panel">
  <AnilloProgreso porcentaje={progreso?.porcentaje ?? 0} {estado} tamano={190} />
  <h2>{titulo}</h2>
  <p class="mensaje">{progreso?.mensaje ?? 'Preparando…'}</p>
  <button class="enlace" onclick={() => (detalles = !detalles)}>{detalles ? 'Ocultar detalles' : 'Ver detalles'}</button>
  {#if detalles}
    <ol class="registro">
      {#each registro as linea}<li>{linea}</li>{/each}
    </ol>
  {/if}
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  h2 {
    margin-top: 8px;
    font-size: 22px;
  }

  .mensaje {
    margin-top: 6px;
    color: var(--texto-suave);
    min-height: 21px;
  }

  .enlace {
    margin-top: 8px;
    border: 0;
    background: none;
    color: var(--cian);
    cursor: pointer;
    font-size: 12px;
  }

  .registro {
    margin: 8px 0 0;
    padding: 8px 12px 8px 30px;
    width: 560px;
    max-height: 110px;
    overflow-y: auto;
    text-align: left;
    font-family: 'Cascadia Mono', Consolas, monospace;
    font-size: 11px;
    color: var(--lavanda);
    background: rgba(0, 0, 0, 0.35);
    border-radius: 10px;
    user-select: text;
  }
</style>
