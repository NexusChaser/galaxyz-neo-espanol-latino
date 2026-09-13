# GALAXYZ neo — Parche al Español Latino 🇲🇽🇦🇷🇨🇴🇵🇪🇨🇱

Traducción no oficial al **español latino** de **GALAXYZ neo** (fuzz), versión de PC (Windows).

## ¿Qué traduce?

Prácticamente todo el texto del juego:

- **Historia completa**: todos los diálogos y escenas (más de 23 000 líneas).
- **Menús e interfaz**, mensajes del sistema y del asistente del menú principal.
- **Personajes**: nombres, trabajos/clases, razas y perfiles.
- **Habilidades, acciones, estados de combate** y elementos.
- **Objetos**, orbes y boletos.
- **Tiendas**: gacha, canjes, tienda de puntos, mejoras temporales y Piedras Galácticas.
- **Misiones**: nombres de partes, capítulos, secciones y los 1388 niveles; condiciones de victoria/derrota.
- **Bonus de inicio de sesión** y caja de regalos.

## Instalación

1. Abre el juego **al menos una vez** para que descargue sus datos, y luego **ciérralo**.
2. Descarga el parche: botón verde **Code → Download ZIP** (arriba en esta página) y descomprímelo.
3. Haz doble clic en **`instalar.bat`**.

Listo. El instalador:

- Hace una **copia de seguridad** de tus archivos originales en
  `%LOCALAPPDATA%\fuzz\galaxyz\data\backup_antes_del_parche\` (solo la primera vez).
- Copia los archivos traducidos en las **3 carpetas** donde el juego los lee.

### Instalación manual

Si prefieres hacerlo a mano, copia **todos** los archivos de la carpeta `epk/` de este repositorio
(reemplazando los existentes) en estas tres carpetas:

```
%LOCALAPPDATA%\fuzz\galaxyz\data\locale\us\epk\
%LOCALAPPDATA%\fuzz\galaxyz\data\epk\
%LOCALAPPDATA%\fuzz\galaxyz\data\root\epk\
```

> Para llegar rápido: pulsa `Win + R`, escribe `%LOCALAPPDATA%\fuzz\galaxyz\data` y presiona Enter.

**Tienen que ir en las tres.** Si solo los pones en una, el juego puede seguir mostrando inglés.

## Desinstalación

Haz doble clic en **`desinstalar.bat`**: restaura los archivos originales desde la copia de seguridad.

Si no tienes la copia de seguridad, borra las carpetas `epk` mencionadas arriba y deja que el juego
vuelva a descargar sus datos.

## Notas

- El juego debe estar en **idioma inglés** (el parche reemplaza los textos en inglés).
- Si el juego se **actualiza** y vuelve a salir texto en inglés, ejecuta `instalar.bat` otra vez.
  Si una actualización cambia el contenido de estos archivos, puede que el parche necesite actualizarse.
- **No modifiques** los `.epk` con un editor de texto: están cifrados y el juego se cierra si están dañados.

## Aviso

Proyecto de fans, sin fines de lucro y sin relación con fuzz ni con los autores de la obra original.
GALAXYZ neo y sus personajes pertenecen a sus respectivos dueños.
