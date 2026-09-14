<script lang="ts">
  import Casilla from '../componentes/Casilla.svelte'
  import SelectorFrecuencia from '../componentes/SelectorFrecuencia.svelte'
  import Tarjeta from '../componentes/Tarjeta.svelte'
  import { formatoMB } from '../lib/api'
  import { asistente, validacionDatosActual } from '../lib/asistente.svelte'

  const vDatos = $derived(validacionDatosActual())
  const ajenos = $derived(vDatos?.estado?.tipo === 'ajenos' ? vDatos.estado.archivos : [])
</script>

<div class="pantalla">
  <header>
    <h2>Opciones</h2>
  </header>

  <div class="columnas">
    <Tarjeta titulo="Actualización automática">
      <div class="bloque">
        <Casilla bind:marcada={asistente.autoActualizar} titulo="Mantener el parche actualizado automáticamente">
          Busca traducciones nuevas en GitHub y las instala solas cuando el juego está cerrado.
        </Casilla>
        {#if asistente.autoActualizar}
          <SelectorFrecuencia bind:horas={asistente.intervalo} opciones={asistente.info?.intervalos ?? [6, 12, 24, 168]} />
        {:else}
          <p class="nota-ambar">
            Sin actualización automática no recibirás correcciones de la traducción por tu cuenta. Podrás activarla
            después desde <strong>Mantenimiento → Cambiar opciones</strong>.
          </p>
        {/if}
      </div>
    </Tarjeta>

    <div class="derecha">
      <Tarjeta titulo="Copia de seguridad">
        {#if ajenos.length > 0}
          <Casilla bind:marcada={asistente.copiaSeguridad} titulo="Hacer copia de seguridad de los archivos que ya hay">
            Se guardará en <code>…\data\backup_antes_del_parche\</code>
            ({ajenos.length} archivos, {formatoMB(ajenos.reduce((s, a) => s + a.tamano, 0))}).
          </Casilla>
          <ul class="lista">
            {#each ajenos.slice(0, 4) as a}<li>{a.ruta}</li>{/each}
            {#if ajenos.length > 4}<li>… y {ajenos.length - 4} más</li>{/if}
          </ul>
        {:else if vDatos?.estado?.tipo === 'parche'}
          <p class="suave">Ya tienes el parche instalado: se actualizará encima. No hace falta copia de seguridad.</p>
        {:else}
          <p class="suave">
            No hace falta: los archivos originales están dentro del juego y no se modifican. Para quitar el parche basta
            con desinstalarlo.
          </p>
        {/if}
      </Tarjeta>

      <Tarjeta titulo="Accesos">
        <Casilla bind:marcada={asistente.accesoDirecto} titulo="Crear acceso directo en el menú Inicio">
          «Parche GALAXYZ neo»: para buscar actualizaciones, reparar o desinstalar.
        </Casilla>
      </Tarjeta>
    </div>
  </div>

  <footer>
    <button class="boton" onclick={() => (asistente.pantalla = 'deteccion')}>← Atrás</button>
    <span class="espacio"></span>
    <button class="boton boton-principal" onclick={() => (asistente.pantalla = 'resumen')}>Siguiente →</button>
  </footer>
</div>

<style>
  .pantalla {
    height: 100%;
    padding: 0 40px 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    animation: entrar 360ms ease both;
  }

  h2 {
    font-size: 24px;
  }

  .columnas {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
    align-items: start;
  }

  .derecha {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .bloque {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .nota-ambar {
    padding: 10px 12px;
    border-radius: 10px;
    background: rgba(255, 179, 71, 0.1);
    border: 1px solid rgba(255, 179, 71, 0.4);
    color: var(--ambar);
    font-size: 12.5px;
  }

  .suave {
    color: var(--texto-suave);
    font-size: 12.5px;
  }

  .lista {
    margin: 8px 0 0 32px;
    padding: 0;
    font-family: 'Cascadia Mono', Consolas, monospace;
    font-size: 11px;
    color: var(--lavanda);
  }

  code {
    font-size: 11.5px;
  }

  footer {
    margin-top: auto;
    display: flex;
    gap: 10px;
  }

  .espacio {
    flex: 1;
  }

  @keyframes entrar {
    from {
      opacity: 0;
      transform: translateX(24px);
    }
  }
</style>
