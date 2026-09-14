<script lang="ts">
  import logo from '../assets/logo.webp'
  import personajes from '../assets/chara.webp'

  let {
    version = '',
    reinstalacion = false,
    alComenzar,
  }: { version?: string; reinstalacion?: boolean; alComenzar: () => void } = $props()
</script>

<div class="bienvenida">
  <div class="texto">
    <img class="logo" src={logo} alt="GALAXYZ neo" />
    <div class="etiqueta">Parche en Español Latino{#if version}&nbsp;·&nbsp;v{version}{/if}</div>
    <p class="subtitulo">Hecho por fans, para fans</p>
    <p class="descripcion">
      Traduce toda la historia, los menús, los personajes, las habilidades, los objetos y las tiendas. Tu juego no se
      modifica: puedes quitar el parche cuando quieras.
    </p>
    <button class="boton boton-principal comenzar" onclick={alComenzar}>
      {reinstalacion ? 'Volver a instalar' : 'Comenzar'}
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
        <path d="M3 8h9M8.5 4l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      </svg>
    </button>
    <p class="aviso-legal">
      Proyecto de fans, gratuito y sin fines de lucro. Sin relación con fuzz ni con Rensuke Oshikiri.<br />
      Las imágenes y personajes pertenecen a sus respectivos dueños.
    </p>
  </div>
  <img class="personajes" src={personajes} alt="" aria-hidden="true" />
</div>

<style>
  .bienvenida {
    position: relative;
    height: 100%;
    display: flex;
    align-items: center;
  }

  .texto {
    position: relative;
    z-index: 2;
    width: 480px;
    padding-left: 56px;
    margin-top: -20px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    animation: entrar 600ms var(--transicion) both;
  }

  .logo {
    width: 400px;
    margin-left: -10px;
    filter: drop-shadow(0 0 24px rgba(190, 120, 255, 0.55));
  }

  .etiqueta {
    margin-top: 18px;
    padding: 6px 16px;
    border-radius: 999px;
    background: var(--rosa);
    color: #fff;
    font-weight: 700;
    font-size: 17px;
    box-shadow: 0 6px 24px var(--rosa-brillo);
  }

  .subtitulo {
    margin-top: 10px;
    color: var(--lavanda);
    font-weight: 600;
  }

  .descripcion {
    margin-top: 12px;
    color: var(--texto-suave);
    max-width: 400px;
  }

  .comenzar {
    margin-top: 24px;
    padding: 13px 28px;
    font-size: 15px;
  }

  .aviso-legal {
    max-width: 330px;
    margin-top: 18px;
    font-size: 11px;
    color: var(--texto-suave);
    opacity: 0.8;
  }

  .personajes {
    position: absolute;
    right: -40px;
    bottom: -70px;
    height: 600px;
    z-index: 1;
    filter: drop-shadow(0 0 30px rgba(140, 90, 255, 0.45));
    mask-image: linear-gradient(to bottom, #000 70%, transparent 100%);
    animation: deslizar 800ms 120ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
  }

  @keyframes entrar {
    from {
      opacity: 0;
      transform: translateY(24px);
    }
  }

  @keyframes deslizar {
    from {
      opacity: 0;
      transform: translateX(60px);
    }
  }
</style>
