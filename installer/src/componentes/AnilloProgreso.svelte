<script lang="ts">
  import phobos1 from '../assets/phobos1.webp'
  import phobos2 from '../assets/phobos2.webp'

  let {
    porcentaje = 0,
    tamano = 200,
    estado = 'activo',
  }: { porcentaje?: number; tamano?: number; estado?: 'activo' | 'ok' | 'error' } = $props()

  const radio = 46
  const circunferencia = 2 * Math.PI * radio
</script>

<div class="anillo {estado}" style="--tam:{tamano}px" role="progressbar" aria-valuenow={Math.round(porcentaje)} aria-valuemin="0" aria-valuemax="100">
  <img class="phobos exterior" src={phobos1} alt="" />
  <img class="phobos interior" src={phobos2} alt="" />
  <svg viewBox="0 0 100 100" class="barra" aria-hidden="true">
    <circle cx="50" cy="50" r={radio} class="pista" />
    <circle
      cx="50"
      cy="50"
      r={radio}
      class="relleno"
      stroke-dasharray={circunferencia}
      stroke-dashoffset={circunferencia * (1 - Math.min(100, Math.max(0, porcentaje)) / 100)}
    />
  </svg>
  <div class="centro">
    {#if estado === 'ok'}
      <span class="icono">✓</span>
    {:else if estado === 'error'}
      <span class="icono">✕</span>
    {:else}
      <span class="cifra">{Math.round(porcentaje)}<small>%</small></span>
    {/if}
  </div>
</div>

<style>
  .anillo {
    position: relative;
    width: var(--tam);
    height: var(--tam);
    display: grid;
    place-items: center;
  }

  .phobos {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  .exterior {
    animation: girar 9s linear infinite;
  }

  .interior {
    inset: 12%;
    width: 76%;
    height: 76%;
    animation: girar 6s linear infinite reverse;
  }

  .barra {
    position: absolute;
    inset: 20%;
    width: 60%;
    height: 60%;
    transform: rotate(-90deg);
  }

  .pista {
    fill: none;
    stroke: rgba(255, 255, 255, 0.08);
    stroke-width: 4;
  }

  .relleno {
    fill: none;
    stroke: var(--rosa);
    stroke-width: 4;
    stroke-linecap: round;
    transition: stroke-dashoffset 300ms ease;
    filter: drop-shadow(0 0 4px var(--rosa-brillo));
  }

  .ok .relleno {
    stroke: var(--cian);
  }

  .error .relleno {
    stroke: var(--rojo);
  }

  .ok .phobos,
  .error .phobos {
    animation-play-state: paused;
    opacity: 0.6;
  }

  .centro {
    position: relative;
    font-family: var(--fuente-titulo);
  }

  .cifra {
    font-size: calc(var(--tam) * 0.16);
    font-weight: 700;
  }

  .cifra small {
    font-size: 0.5em;
    color: var(--texto-suave);
    margin-left: 1px;
  }

  .icono {
    font-size: calc(var(--tam) * 0.2);
    font-weight: 800;
  }

  .ok .icono {
    color: var(--cian);
  }

  .error .icono {
    color: var(--rojo);
  }

  @keyframes girar {
    to {
      transform: rotate(360deg);
    }
  }
</style>
