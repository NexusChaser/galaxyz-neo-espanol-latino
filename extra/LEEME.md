# extra/

Archivos del parche que **no** son `.epk` de texto, como las **imágenes traducidas**.

Cada archivo se instala en la **misma ruta** dentro de `%LOCALAPPDATA%\fuzz\galaxyz\data\`:

```
extra/res/gui/textures/TEST/logo.webp  →  %LOCALAPPDATA%\fuzz\galaxyz\data\res\gui\textures\TEST\logo.webp
```

La ruta sale del nombre del archivo dentro del OBB cambiando `#` por `/`
(`res#gui#textures#TEST#logo.webp` → `res/gui/textures/TEST/logo.webp`).

Lo usan el instalador, `instalar.bat` y `desinstalar.bat`. Explicación completa en
[`docs/INSTALADOR.md`](../docs/INSTALADOR.md#añadir-imágenes-traducidas).

Este `LEEME.md` no se instala.
