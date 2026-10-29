import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

/**
 * La transcripción usa solo el motor WASM básico de onnxruntime. Las variantes asyncify/JSPI/JSEP (WebGPU) no se usan
 * (se fija `wasmPaths` en whisperWorker.js) y pesan ~27 MB: se quitan del paquete.
 */
const dropUnusedOrt = () => ({
  name: "drop-unused-ort",
  generateBundle(_, bundle) {
    for (const key of Object.keys(bundle)) if (/ort-wasm.*(asyncify|jspi|jsep)/.test(key)) delete bundle[key];
  },
});

export default defineConfig({
  plugins: [react(), dropUnusedOrt()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Evita EBUSY del FSWatcher mientras Tauri compila
      ignored: ["**/src-tauri/**"],
    },
  },
});
