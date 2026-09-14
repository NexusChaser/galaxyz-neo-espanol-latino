<script lang="ts">
  import type { Snippet } from 'svelte'

  let {
    titulo,
    abierto = $bindable(false),
    children,
    acciones,
  }: { titulo: string; abierto?: boolean; children: Snippet; acciones: Snippet } = $props()

  function tecla(e: KeyboardEvent) {
    if (e.key === 'Escape') abierto = false
  }
</script>

<svelte:window onkeydown={abierto ? tecla : undefined} />

{#if abierto}
  <div class="capa" role="presentation" onclick={() => (abierto = false)}>
    <div class="dialogo" role="dialog" aria-modal="true" aria-label={titulo} tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <h3>{titulo}</h3>
      <div class="cuerpo">{@render children()}</div>
      <div class="acciones">{@render acciones()}</div>
    </div>
  </div>
{/if}

<style>
  .capa {
    position: absolute;
    inset: 0;
    z-index: 50;
    background: rgba(5, 2, 20, 0.6);
    backdrop-filter: blur(4px);
    display: grid;
    place-items: center;
    animation: aparecer 160ms ease both;
  }

  .dialogo {
    width: 440px;
    max-width: calc(100% - 40px);
    background: #1a0c40;
    border: 1px solid var(--lavanda-borde);
    border-radius: var(--radio-grande);
    padding: 22px 24px;
    box-shadow: var(--sombra);
    animation: subir 200ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
  }

  h3 {
    font-size: 18px;
    margin-bottom: 10px;
  }

  .cuerpo {
    color: var(--texto-suave);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .acciones {
    margin-top: 20px;
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  @keyframes aparecer {
    from {
      opacity: 0;
    }
  }

  @keyframes subir {
    from {
      opacity: 0;
      transform: translateY(12px) scale(0.98);
    }
  }
</style>
