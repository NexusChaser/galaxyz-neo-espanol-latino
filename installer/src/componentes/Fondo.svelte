<script lang="ts">
  import fondo from '../assets/title.webp'

  // Estrellas con posición y ritmo fijos (semilla), para que no "salten" entre recargas.
  let semilla = 7
  const azar = () => ((semilla = (semilla * 16807) % 2147483647) - 1) / 2147483646
  const estrellas = Array.from({ length: 70 }, () => ({
    x: azar() * 100,
    y: azar() * 100,
    tam: 1 + azar() * 2,
    retraso: azar() * 6,
    duracion: 3 + azar() * 4,
  }))
</script>

<div class="fondo" aria-hidden="true">
  <img class="imagen" src={fondo} alt="" />
  <div class="brillo rosa"></div>
  <div class="brillo cian"></div>
  {#each estrellas as e}
    <span
      class="estrella"
      style="left:{e.x}%;top:{e.y}%;width:{e.tam}px;height:{e.tam}px;animation-delay:{e.retraso}s;animation-duration:{e.duracion}s"
    ></span>
  {/each}
  <div class="velo"></div>
</div>

<style>
  .fondo {
    position: absolute;
    inset: 0;
    overflow: hidden;
    background: var(--fondo);
  }

  .imagen {
    position: absolute;
    inset: -4%;
    width: 108%;
    height: 108%;
    object-fit: cover;
    opacity: 0.85;
    animation: deriva 40s ease-in-out infinite alternate;
  }

  .brillo {
    position: absolute;
    border-radius: 50%;
    filter: blur(90px);
  }

  .rosa {
    width: 420px;
    height: 420px;
    right: -80px;
    top: -60px;
    background: rgba(255, 79, 150, 0.28);
  }

  .cian {
    width: 380px;
    height: 380px;
    left: -120px;
    bottom: -160px;
    background: rgba(63, 208, 255, 0.18);
  }

  .estrella {
    position: absolute;
    border-radius: 50%;
    background: #fff;
    opacity: 0.2;
    animation: parpadeo ease-in-out infinite;
  }

  /* Oscurece el lado izquierdo, donde va el texto. */
  .velo {
    position: absolute;
    inset: 0;
    background:
      linear-gradient(90deg, rgba(11, 7, 38, 0.92) 0%, rgba(11, 7, 38, 0.55) 55%, rgba(11, 7, 38, 0.2) 100%),
      linear-gradient(0deg, rgba(11, 7, 38, 0.7) 0%, transparent 35%);
  }

  @keyframes parpadeo {
    0%,
    100% {
      opacity: 0.15;
    }
    50% {
      opacity: 0.9;
    }
  }

  @keyframes deriva {
    from {
      transform: scale(1) translate(0, 0);
    }
    to {
      transform: scale(1.04) translate(-1.5%, 1%);
    }
  }
</style>
