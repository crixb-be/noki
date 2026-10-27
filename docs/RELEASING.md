# Publishing a release / Publicar una versión

Noki can update itself from a `latest.json` file that you publish with each version (for example in a GitHub Release).
Noki puede actualizarse sola desde un archivo `latest.json` que publicas con cada versión (por ejemplo en una Release de GitHub).

## One-time setup / Una sola vez

1. Create your **updater signing key** (keep it secret and back it up — without it installed apps cannot accept your updates):
   Crea tu **clave de firma** de actualizaciones (guárdala en secreto y haz copia; sin ella las apps instaladas no aceptan tus actualizaciones):

   ```powershell
   npx tauri signer generate -w $env:USERPROFILE\.tauri\noki.key
   ```

2. Copy the **public** key it prints into `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`.
   Copia la clave **pública** que imprime en `plugins.updater.pubkey` de `src-tauri/tauri.conf.json`.

> ⚠️ **Never commit the private key** (`noki.key`). It lives outside the repository on purpose.
> ⚠️ **Nunca subas la clave privada** (`noki.key`). Vive fuera del repositorio a propósito.

## For each version / En cada versión

1. Bump the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`, and add a line to `CHANGELOG.md`.
   Sube la versión en los tres archivos y añade una línea a `CHANGELOG.md`.
2. Build the signed installer / Compila el instalador firmado:

   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts\compilar.ps1
   ```

3. Prepare the files to upload (creates `publicar\` with the installer and `latest.json`) /
   Prepara los archivos (crea `publicar\` con el instalador y `latest.json`):

   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts\publicar-actualizacion.ps1 `
     -BaseUrl "https://github.com/crixb-be/noki/releases/download/v1.0.0" -Notas "What's new"
   ```

4. Create a GitHub Release tagged `v1.0.0` and attach the two files from `publicar\`.
   Crea una Release en GitHub con la etiqueta `v1.0.0` y adjunta los dos archivos de `publicar\`.
5. Noki already looks for updates at
   `https://github.com/crixb-be/noki/releases/latest/download/latest.json` (Settings → About → Updates), so users get the new version automatically.
   Noki ya busca actualizaciones en esa dirección (Ajustes → Acerca de → Actualizaciones), así que quien la use recibe la nueva versión sola.

The app only installs installers signed with your key **and** whose signed version matches the announced one.
La app solo instala instaladores firmados con tu clave **y** cuya versión firmada coincide con la anunciada.
