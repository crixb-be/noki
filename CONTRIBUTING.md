# Contributing to Noki / Contribuir a Noki

Thanks for your interest! / ¡Gracias por tu interés!

## Report a bug / Reportar un error

Open an [issue](../../issues/new/choose) and include: / Abre un [issue](../../issues/new/choose) e incluye:

- Noki version (Settings → About) / versión de Noki (Ajustes → Acerca de)
- Windows version / versión de Windows
- What you did, what you expected and what happened / qué hiciste, qué esperabas y qué pasó
- A screenshot if it helps / una captura si ayuda

**Never paste API keys.** / **Nunca pegues claves de API.**

## Suggest an idea / Proponer una idea

Open an issue with the *Feature request* template. Small, focused ideas are easier to build. /
Abre un issue con la plantilla *Feature request*. Las ideas pequeñas y concretas son más fáciles de hacer.

## Send code / Enviar código

1. Fork and create a branch. / Haz un fork y crea una rama.
2. `npm install`, then `npm run tauri dev`.
3. Run the tests: `cd src-tauri && cargo test --lib`.
4. Keep changes focused, match the surrounding style, and describe what you tested. /
   Mantén los cambios acotados, sigue el estilo del código cercano y describe qué probaste.
5. Open a pull request. / Abre un pull request.

### Notes
- UI text is written in Spanish in the code; English lives in `src/lib/en.js`. If you add or change a text, add its English there. /
  Los textos de la interfaz están en español en el código; el inglés está en `src/lib/en.js`. Si añades un texto, agrega allí su traducción.
- Noki is licensed under **GPL-3.0-or-later**. By contributing you agree your contribution is released under the same license. /
  Noki usa la licencia **GPL-3.0-o-posterior**. Al contribuir aceptas que tu aporte se publique con la misma licencia.
- Do not copy code or assets from projects with incompatible licenses. / No copies código ni recursos de proyectos con licencias incompatibles.

## Questions / Preguntas

For questions, ideas or to show your setup, use [Discussions](../../discussions). / Para dudas, ideas o para enseñar tu configuración, usa [Discussions](../../discussions).
