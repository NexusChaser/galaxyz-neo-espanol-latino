<script lang="ts">
  import type { Snippet } from 'svelte'

  let {
    titulo = '',
    estado = 'neutro',
    children,
  }: { titulo?: string; estado?: 'ok' | 'aviso' | 'error' | 'neutro'; children: Snippet } = $props()

  const icono = { ok: '✓', aviso: '!', error: '✕', neutro: '' }
</script>

<section class="tarjeta {estado}">
  {#if titulo}
    <h3>
      {#if icono[estado]}<span class="icono">{icono[estado]}</span>{/if}
      {titulo}
    </h3>
  {/if}
  {@render children()}
</section>

<style>
  .tarjeta {
    background: var(--morado-vidrio);
    backdrop-filter: var(--desenfoque);
    border: 1px solid var(--lavanda-borde);
    border-radius: var(--radio);
    padding: 14px 16px;
    box-shadow: var(--sombra);
  }

  h3 {
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .icono {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 11px;
    font-weight: 800;
    color: var(--fondo);
    background: var(--lavanda);
  }

  .ok .icono {
    background: var(--cian);
  }

  .aviso {
    border-color: rgba(255, 179, 71, 0.45);
  }

  .aviso .icono {
    background: var(--ambar);
  }

  .error {
    border-color: rgba(255, 107, 122, 0.5);
  }

  .error .icono {
    background: var(--rojo);
  }
</style>
