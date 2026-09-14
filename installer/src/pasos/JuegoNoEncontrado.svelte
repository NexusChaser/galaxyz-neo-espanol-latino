<script lang="ts">
  import tierra from '../assets/kumo1.webp'
  import { asistente, cargarDeteccion } from '../lib/asistente.svelte'

  let buscando = $state(false)

  async function volverABuscar() {
    buscando = true
    await cargarDeteccion()
    buscando = false
    if (asistente.deteccion?.juego) asistente.pantalla = 'deteccion'
  }

  function elegirAMano() {
    asistente.modoJuego = 'manual'
    asistente.pantalla = 'deteccion'
  }

  function instalarIgualmente() {
    asistente.sinJuego = true
    asistente.pantalla = 'deteccion'
  }
</script>

<div class="pantalla">
  <img class="tierra" src={tierra} alt="" aria-hidden="true" />
  <h2>No encontramos GALAXYZ neo en esta PC</h2>
  <ul>
    <li>Si lo tienes en Steam, comprueba que esté instalado y vuelve a buscar.</li>
    <li>Si está en otra carpeta, elígela a mano.</li>
    <li>También puedes instalar el parche igualmente: no depende de la carpeta del juego.</li>
  </ul>
  <div class="acciones">
    <button class="boton" onclick={volverABuscar} disabled={buscando}>{buscando ? 'Buscando…' : 'Volver a buscar'}</button>
    <button class="boton" onclick={elegirAMano}>Elegir carpeta del juego</button>
    <button class="boton boton-principal" onclick={instalarIgualmente}>Instalar igualmente</button>
  </div>
  {#if buscando === false && asistente.deteccion && !asistente.deteccion.juego}
    <p class="suave">Última búsqueda: sin resultados en Steam ni en el registro de Windows.</p>
  {/if}
  <button class="boton atras" onclick={() => (asistente.pantalla = 'bienvenida')}>← Atrás</button>
</div>

<style>
  .pantalla {
    height: 100%;
    padding: 0 40px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    animation: entrar 360ms ease both;
  }

  .tierra {
    width: 150px;
    height: 150px;
    margin-top: 4px;
    border-radius: 50%;
    box-shadow:
      0 0 40px rgba(127, 231, 255, 0.35),
      inset -20px -10px 40px rgba(0, 0, 0, 0.6);
    animation: girar 60s linear infinite;
  }

  h2 {
    margin-top: 18px;
    font-size: 22px;
  }

  ul {
    margin: 14px 0 0;
    padding: 0;
    list-style: none;
    color: var(--texto-suave);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .acciones {
    margin-top: 22px;
    display: flex;
    gap: 10px;
  }

  .suave {
    margin-top: 12px;
    font-size: 12px;
    color: var(--texto-suave);
  }

  .atras {
    margin-top: auto;
    align-self: flex-start;
  }

  @keyframes girar {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes entrar {
    from {
      opacity: 0;
      transform: translateY(16px);
    }
  }
</style>
