<p align="center">
  <img src=".github/banner.svg" alt="GALAXYZ neo — Parche de traducción al Español Latino" width="100%">
</p>

<p align="center">
  <a href="https://github.com/NexusChaser/galaxyz-neo-espanol-latino/releases/latest"><img src="https://img.shields.io/github/v/release/NexusChaser/galaxyz-neo-espanol-latino?label=versi%C3%B3n&style=for-the-badge&color=8a5cf6" alt="Versión"></a>
  <img src="https://img.shields.io/badge/idioma-Espa%C3%B1ol%20Latino-ff5fd2?style=for-the-badge" alt="Idioma">
  <img src="https://img.shields.io/badge/plataforma-Windows-3fd0ff?style=for-the-badge&logo=windows&logoColor=white" alt="Plataforma">
  <img src="https://img.shields.io/badge/hecho%20por-fans%20para%20fans-ffb347?style=for-the-badge" alt="Fans para fans">
</p>

<p align="center">
  <a href="https://github.com/NexusChaser/galaxyz-neo-espanol-latino/releases/latest/download/GALAXYZ-neo-Espanol-Latino.zip"><b>⬇️ Descargar parche</b></a>
  &nbsp;·&nbsp;
  <a href="#-instalación"><b>📦 Instalación</b></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/NexusChaser/galaxyz-neo-espanol-latino/issues/new/choose"><b>📝 Reportar un error</b></a>
</p>

---

## 🌌 Sobre el parche

Traducción **no oficial** al **español latino** de **GALAXYZ neo**, versión de PC (Windows).
Un proyecto **hecho por fans, para fans**.

### ✅ Qué está traducido

**Todo el texto editable del juego**, es decir, todo lo que el juego muestra como texto:

| | Contenido |
|---|---|
| 🎬 | **Historia completa**: todos los diálogos y escenas (más de 23 000 líneas) |
| 🧭 | **Menús e interfaz**, mensajes del sistema y diálogos del asistente del menú principal |
| 👥 | **Personajes**: nombres, trabajos/clases, especies y perfiles |
| ⚔️ | **Combate**: habilidades, acciones, estados alterados, elementos y tipos de ataque |
| 🎒 | **Objetos**, orbes y boletos |
| 🛒 | **Tiendas**: gacha, canjes, tienda de puntos, mejoras temporales y Piedras Galácticas |
| 🗺️ | **Misiones**: partes, capítulos, secciones, los 1388 niveles y sus condiciones de victoria/derrota |
| 🎁 | **Bonus de inicio de sesión** y caja de regalos |

### 🖼️ Qué falta por traducir

Los **textos que forman parte de una imagen** (texto "pegado" en ilustraciones, logos, carteles,
botones dibujados, etc.) **todavía no están traducidos**. Eso llegará en **futuras actualizaciones**.

---

## 📦 Instalación

1. Abre el juego **al menos una vez** para que descargue sus datos y luego **ciérralo**.
2. **[Descarga el parche](https://github.com/NexusChaser/galaxyz-neo-espanol-latino/releases/latest/download/GALAXYZ-neo-Espanol-Latino.zip)** y descomprime el ZIP.
3. Haz doble clic en **`instalar.bat`**.

¡Y listo! El instalador:

- 💾 Guarda una **copia de seguridad** de tus archivos originales en
  `%LOCALAPPDATA%\fuzz\galaxyz\data\backup_antes_del_parche\` (solo la primera vez).
- 📂 Copia los archivos traducidos en las **3 carpetas** donde el juego los lee.

> [!WARNING]
> **¿El juego se cierra al arrancar las primeras veces? ¡No te rindas!**
>
> Es **normal** que la primera vez (o las primeras veces) que abras el juego con el parche **se cierre
> solo mientras carga en la pantalla de inicio**. **Vuelve a abrirlo.** Suele arrancar bien después de
> **3 o 4 intentos como máximo**. Todavía no sabemos por qué pasa, pero una vez que arranca funciona
> con normalidad.

<details>
<summary><b>🛠️ Instalación manual (sin el .bat)</b></summary>

<br>

Copia **todos** los archivos de la carpeta `epk/` (reemplazando los que ya existen) en estas **tres** carpetas:

```
%LOCALAPPDATA%\fuzz\galaxyz\data\locale\us\epk\
%LOCALAPPDATA%\fuzz\galaxyz\data\epk\
%LOCALAPPDATA%\fuzz\galaxyz\data\root\epk\
```

Para llegar rápido: pulsa `Win + R`, escribe `%LOCALAPPDATA%\fuzz\galaxyz\data` y presiona Enter.

**Tienen que ir en las tres.** Si solo los pones en una, el juego puede seguir mostrando inglés.

</details>

<details>
<summary><b>↩️ Desinstalación</b></summary>

<br>

Haz doble clic en **`desinstalar.bat`**: restaura los archivos originales desde la copia de seguridad.

Si no tienes la copia de seguridad, borra las carpetas `epk` indicadas arriba y deja que el juego
vuelva a descargar sus datos.

</details>

<details>
<summary><b>❓ Notas y problemas comunes</b></summary>

<br>

- El juego debe estar en **inglés**: el parche reemplaza los textos en inglés.
- Si el juego se **actualiza** y vuelve a salir texto en inglés, ejecuta `instalar.bat` otra vez.
- **No abras ni edites** los `.epk` con un editor de texto: están cifrados y el juego se cierra si están dañados.

</details>

---

## 🐞 ¿Encontraste un error?

La traducción es enorme y **hay fallas en algunas partes**: frases raras, textos cortados, algo que
quedó en inglés, un nombre que no coincide… **Cualquier cosa rara que veas, avísanos** para poder
corregirla.

👉 **[Abre un reporte aquí](https://github.com/NexusChaser/galaxyz-neo-espanol-latino/issues/new/choose)**.
Si puedes, dinos **dónde aparece** (menú, capítulo, personaje) y adjunta una **captura de pantalla**.

---

## 💜 Agradecimientos

- **[kurikomoe](https://github.com/kurikomoe)**, creador de **[FSNr_tools](https://github.com/kurikomoe/FSNr_tools)**.
  Esa herramienta sirvió para descifrar y volver a cifrar los archivos del juego, ya que usa el mismo
  motor. **Sin ella este parche no habría sido posible.** ¡Muchas gracias!
- A todos los que prueban el parche y reportan errores.

---

## ⚖️ Aviso legal

Este es un **proyecto de fans para fans**, **gratuito** y **sin fines de lucro**.

**No tenemos ninguna relación con fuzz ni con Rensuke Oshikiri**, ni contamos con su respaldo.
GALAXYZ neo, sus personajes y todo su contenido pertenecen a sus respectivos dueños.
Si te gusta el juego, apoya a sus creadores.
