<script lang="ts">
  let { pasos, actual }: { pasos: string[]; actual: number } = $props()
</script>

<ol class="pasos" aria-label="Pasos de la instalación">
  {#each pasos as paso, i}
    <li class:hecho={i < actual} class:actual={i === actual} aria-current={i === actual ? 'step' : undefined}>
      <span class="punto">{i < actual ? '✓' : i + 1}</span>
      <span class="nombre">{paso}</span>
    </li>
  {/each}
</ol>

<style>
  .pasos {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    gap: 6px;
    align-items: center;
  }

  li {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 4px 10px 4px 4px;
    border-radius: 999px;
    color: var(--texto-suave);
    font-size: 12px;
    transition:
      color var(--transicion),
      background var(--transicion);
  }

  li + li::before {
    content: '';
    width: 14px;
    height: 1px;
    background: var(--lavanda-borde);
    margin-right: 6px;
    margin-left: -4px;
  }

  .punto {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 11px;
    font-weight: 700;
    border: 1px solid var(--lavanda-borde);
  }

  .actual {
    color: var(--texto);
    background: rgba(255, 255, 255, 0.07);
  }

  .actual .punto {
    background: var(--rosa);
    border-color: transparent;
    color: #fff;
    box-shadow: 0 0 14px var(--rosa-brillo);
  }

  .hecho .punto {
    border-color: var(--cian);
    color: var(--cian);
  }
</style>
